//! Terminal embarqué : exécute un binaire dans un vrai pseudo-terminal (PTY) et
//! en émule l'écran avec `vt100`, pour la vue `:test`.
//!
//! Le binaire tourne attaché à un PTY (via `portable-pty`), donc il se comporte
//! comme devant un vrai terminal : sortie non bufferisée, mode brut, séquences
//! ANSI, applications type `curses`. Un thread lit la sortie du PTY et la pousse
//! dans un canal ; [`Terminal::pump`] l'injecte dans le parseur `vt100`. Les
//! frappes clavier sont encodées en octets et renvoyées à l'entrée du binaire.

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

/// Un binaire en cours d'exécution dans un PTY, avec son écran émulé.
pub struct Terminal {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output: Receiver<Vec<u8>>,
    parser: vt100::Parser,
    size: (u16, u16),
    exit: Option<u32>,
    /// Ligne de commande affichée dans l'en-tête de la vue test.
    pub cmdline: String,
}

fn to_io<E: std::fmt::Display>(e: E) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

impl Terminal {
    /// Lance `program args` dans `cwd`, attaché à un PTY de `rows`×`cols`.
    pub fn spawn(
        program: &Path,
        args: &[String],
        cwd: &Path,
        rows: u16,
        cols: u16,
    ) -> std::io::Result<Self> {
        let rows = rows.max(1);
        let cols = cols.max(1);
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(to_io)?;

        let mut cmd = CommandBuilder::new(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.cwd(cwd);

        let child = pair.slave.spawn_command(cmd).map_err(to_io)?;
        let mut reader = pair.master.try_clone_reader().map_err(to_io)?;
        let writer = pair.master.take_writer().map_err(to_io)?;
        drop(pair.slave); // libère l'esclave pour que l'EOF arrive à la sortie du binaire

        let (tx, output) = mpsc::channel::<Vec<u8>>();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        let cmdline = format!("{} {}", program.display(), args.join(" "))
            .trim()
            .to_string();

        Ok(Self {
            master: pair.master,
            writer,
            child,
            output,
            parser: vt100::Parser::new(rows, cols, 0),
            size: (rows, cols),
            exit: None,
            cmdline,
        })
    }

    /// Injecte la sortie disponible dans l'émulateur et relève la fin éventuelle.
    pub fn pump(&mut self) {
        while let Ok(chunk) = self.output.try_recv() {
            self.parser.process(&chunk);
        }
        if self.exit.is_none() {
            if let Ok(Some(status)) = self.child.try_wait() {
                self.exit = Some(status.exit_code());
            }
        }
    }

    /// Redimensionne le PTY et l'écran émulé (sans effet si inchangé).
    pub fn resize(&mut self, rows: u16, cols: u16) {
        let (rows, cols) = (rows.max(1), cols.max(1));
        if (rows, cols) == self.size {
            return;
        }
        self.size = (rows, cols);
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        self.parser.set_size(rows, cols);
    }

    /// Transmet une frappe clavier à l'entrée standard du binaire.
    pub fn send_key(&mut self, key: KeyEvent) {
        let bytes = encode_key(key);
        if bytes.is_empty() {
            return;
        }
        let _ = self.writer.write_all(&bytes);
        let _ = self.writer.flush();
    }

    /// Écran émulé courant, à rendre par l'UI.
    pub fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    /// Vrai si le binaire s'est terminé.
    pub fn finished(&self) -> bool {
        self.exit.is_some()
    }

    /// Code de sortie du binaire, une fois terminé.
    pub fn exit_code(&self) -> Option<u32> {
        self.exit
    }

    /// Tue le binaire s'il tourne encore (à la fermeture de la vue test).
    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }
}

/// Encode une frappe crossterm en octets pour l'entrée du binaire.
fn encode_key(key: KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char(c) if ctrl => {
            // Ctrl-A..Ctrl-Z et quelques voisins -> codes de contrôle.
            let b = (c.to_ascii_uppercase() as u8).wrapping_sub(b'@');
            vec![b & 0x1f]
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf).as_bytes().to_vec()
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn encodes_common_keys() {
        let plain = |code| KeyEvent::new(code, KeyModifiers::NONE);
        assert_eq!(encode_key(plain(KeyCode::Char('a'))), b"a");
        assert_eq!(encode_key(plain(KeyCode::Enter)), b"\r");
        assert_eq!(encode_key(plain(KeyCode::Up)), b"\x1b[A");
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(encode_key(ctrl_c), vec![0x03]);
    }

    #[test]
    fn runs_a_binary_in_a_pty_and_captures_output() {
        let dir = tempfile::tempdir().unwrap();
        let mut term = Terminal::spawn(
            Path::new("/bin/echo"),
            &["hello42".to_string()],
            dir.path(),
            24,
            80,
        )
        .expect("spawn echo");

        // Laisse le temps au binaire d'écrire puis de se terminer.
        let mut waited = 0;
        while waited < 3000 {
            term.pump();
            if term.finished() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
            waited += 10;
        }
        // Draine les derniers octets après la sortie.
        for _ in 0..20 {
            term.pump();
            thread::sleep(Duration::from_millis(5));
        }

        assert!(term.finished(), "le binaire aurait dû se terminer");
        let screen = term.screen();
        let row0: String = (0..80)
            .filter_map(|c| screen.cell(0, c).map(|cell| cell.contents()))
            .collect();
        assert!(row0.contains("hello42"), "écran ligne 0: {row0:?}");
    }
}
