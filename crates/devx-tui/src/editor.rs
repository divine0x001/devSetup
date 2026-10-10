//! Éditeur de texte : tampon + curseur, colonnes en `char` (UTF-8 sûr).

use std::fs;
use std::path::{Path, PathBuf};

use devx_core::DevxError;

/// Tampon éditable.
///
/// Les colonnes du curseur sont des indices **de caractères** (`char`), jamais
/// des indices d'octets : l'édition est sûre vis-à-vis de l'UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    /// Contenu découpé en lignes (jamais vide : au moins une chaîne).
    pub lines: Vec<String>,
    /// Ligne du curseur.
    pub cursor_row: usize,
    /// Colonne du curseur (indice de `char` dans la ligne).
    pub cursor_col: usize,
    /// Fichier associé, le cas échéant.
    pub path: Option<PathBuf>,
    /// Vrai si le tampon diffère de l'état sur disque.
    pub dirty: bool,
}

impl Buffer {
    /// Tampon vide, sans fichier associé.
    pub fn empty() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            path: None,
            dirty: false,
        }
    }

    /// Tampon vide associé à un chemin (pour créer un nouveau fichier).
    pub fn empty_with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            path: Some(path.into()),
            dirty: false,
        }
    }

    /// Charge un fichier dans un nouveau tampon.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DevxError> {
        let path = path.as_ref().to_path_buf();
        let raw = fs::read_to_string(&path)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;
        let mut lines: Vec<String> = raw.lines().map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Ok(Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            path: Some(path),
            dirty: false,
        })
    }

    /// Sauvegarde le tampon sur disque (échec si aucun fichier associé).
    pub fn save(&mut self) -> Result<(), DevxError> {
        let Some(path) = &self.path else {
            return Err(DevxError::message(
                "aucun fichier associé (:e <fichier> d'abord)",
            ));
        };
        let mut content = self.lines.join("\n");
        content.push('\n');
        fs::write(path, content)
            .map_err(|err| DevxError::io(format!("écriture de {}: {err}", path.display())))?;
        self.dirty = false;
        Ok(())
    }

    /// Position du curseur en coordonnées affichables (ligne:colonne 1-based).
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row + 1, self.cursor_col + 1)
    }

    /// Insère un caractère à la position du curseur.
    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_row];
        let mut chars: Vec<char> = line.chars().collect();
        chars.insert(self.cursor_col, c);
        *line = chars.into_iter().collect();
        self.cursor_col += 1;
        self.dirty = true;
    }

    /// Coupe la ligne courante à la colonne du curseur et insère une nouvelle
    /// ligne avec la partie droite.
    pub fn insert_newline(&mut self) {
        let right = self.split_at_cursor();
        self.lines.insert(self.cursor_row + 1, right);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.dirty = true;
    }

    /// Efface le caractère précédant le curseur ; en début de ligne, fusionne
    /// avec la ligne précédente.
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_col - 1);
            *line = chars.into_iter().collect();
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            let merged = format!("{}{}", self.lines[self.cursor_row], current);
            self.lines[self.cursor_row] = merged;
            self.cursor_col = self.line_len(self.cursor_row);
        }
        self.dirty = true;
    }

    /// Efface le caractère sous le curseur ; en fin de ligne, fusionne avec la
    /// ligne suivante.
    pub fn delete(&mut self) {
        let len = self.line_len(self.cursor_row);
        if self.cursor_col < len {
            let line = &mut self.lines[self.cursor_row];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_col);
            *line = chars.into_iter().collect();
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
        }
        self.dirty = true;
    }

    /// Déplace le curseur d'une ligne vers le haut.
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_col();
        }
    }

    /// Déplace le curseur d'une ligne vers le bas.
    pub fn move_down(&mut self) {
        let last = self.lines.len() - 1;
        if self.cursor_row < last {
            self.cursor_row += 1;
            self.clamp_col();
        }
    }

    /// Déplace le curseur d'un caractère vers la gauche.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
        }
    }

    /// Déplace le curseur d'un caractère vers la droite.
    pub fn move_right(&mut self) {
        let last = self.lines.len() - 1;
        if self.cursor_col < self.line_len(self.cursor_row) {
            self.cursor_col += 1;
        } else if self.cursor_row < last {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Place le curseur en début de ligne.
    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Place le curseur en fin de ligne.
    pub fn move_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_row);
    }

    /// Coupe la ligne courante à la colonne du curseur ; le curseur garde la
    /// partie gauche, on renvoie la partie droite.
    fn split_at_cursor(&mut self) -> String {
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        let left: String = chars[..self.cursor_col].iter().collect();
        let right: String = chars[self.cursor_col..].iter().collect();
        self.lines[self.cursor_row] = left;
        right
    }

    fn line_len(&self, row: usize) -> usize {
        self.lines[row].chars().count()
    }

    fn clamp_col(&mut self) {
        self.cursor_col = self.cursor_col.min(self.line_len(self.cursor_row));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn empty_buffer_has_one_line() {
        let buffer = Buffer::empty();
        assert_eq!(buffer.lines, vec![String::new()]);
        assert_eq!(buffer.cursor_row, 0);
        assert_eq!(buffer.cursor_col, 0);
        assert!(!buffer.dirty);
        assert!(buffer.path.is_none());
    }

    #[test]
    fn insert_char_at_cursor() {
        let mut b = Buffer::empty();
        b.insert_char('a');
        b.insert_char('b');
        b.move_left();
        b.insert_char('X');
        assert_eq!(b.lines[0], "aXb");
        assert_eq!(b.cursor_col, 2);
        assert!(b.dirty);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut b = Buffer::empty();
        for c in "ab".chars() {
            b.insert_char(c);
        }
        b.move_left();
        b.insert_newline();
        assert_eq!(b.lines, vec!["a", "b"]);
        assert_eq!((b.cursor_row, b.cursor_col), (1, 0));
    }

    #[test]
    fn backspace_mid_line_and_merge() {
        let mut b = Buffer::empty();
        for c in "abcd".chars() {
            b.insert_char(c);
        }
        b.move_left();
        b.backspace();
        assert_eq!(b.lines[0], "abd");

        // En début de ligne : fusion avec la précédente.
        let mut b = Buffer::empty();
        b.insert_char('x');
        b.insert_newline();
        b.insert_char('y');
        b.move_home();
        b.backspace();
        assert_eq!(b.lines, vec!["xy"]);
        assert_eq!(b.cursor_row, 0);
    }

    #[test]
    fn delete_char_and_merge_next() {
        let mut b = Buffer::empty();
        for c in "abcd".chars() {
            b.insert_char(c);
        }
        b.move_home();
        b.delete();
        assert_eq!(b.lines[0], "bcd");

        // En fin de ligne : fusion avec la suivante.
        let mut b = Buffer::empty();
        b.insert_char('a');
        b.insert_newline();
        b.insert_char('b');
        b.move_up();
        b.move_end();
        b.delete();
        assert_eq!(b.lines, vec!["ab"]);
    }

    #[test]
    fn cursor_movements_respect_bounds() {
        let mut b = Buffer::empty();
        for c in "hello".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        b.insert_char('w');
        b.move_end();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));

        // En bas : déjà sur la dernière ligne.
        b.move_down();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));

        // En haut : colonne bornée à la longueur de la ligne.
        b.move_up();
        b.move_up();
        assert_eq!(b.cursor_row, 0);
        assert_eq!(b.cursor_col, 1);

        // Droite au-delà de la fin de ligne : passe à la ligne suivante.
        b.move_end();
        b.move_right();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 0));
        b.move_right();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));
        b.move_right();
        assert_eq!((b.cursor_row, b.cursor_col), (1, 1));
    }

    #[test]
    fn moving_down_clamps_column_to_line_length() {
        let mut b = Buffer::empty();
        for c in "very long line".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        b.insert_char('s');
        b.move_up();
        b.move_end();
        b.move_down();
        assert_eq!(b.cursor_row, 1);
        assert_eq!(b.cursor_col, 1);
    }

    #[test]
    fn home_and_end() {
        let mut b = Buffer::empty();
        for c in "abc".chars() {
            b.insert_char(c);
        }
        b.move_home();
        assert_eq!(b.cursor_col, 0);
        b.move_end();
        assert_eq!(b.cursor_col, 3);
    }

    #[test]
    fn utf8_columns_are_char_based() {
        let mut b = Buffer::empty();
        for c in "héllo 🙂".chars() {
            b.insert_char(c);
        }
        b.move_home();
        b.insert_char('X');
        assert_eq!(b.lines[0], "Xhéllo 🙂");
        // Colonne en caractères (pas d'octets).
        b.move_end();
        assert_eq!(b.cursor_col, "Xhéllo 🙂".chars().count());
        b.backspace();
        b.move_end();
        assert_eq!(b.cursor_col, "Xhéllo ".chars().count());
    }

    #[test]
    fn load_and_save_round_trip() {
        let dir = env::temp_dir();
        let path = dir.join(format!("devx-tui-test-{}.txt", std::process::id()));
        fs::write(&path, "ligne une\nligne deux\n").unwrap();

        let mut b = Buffer::load(&path).unwrap();
        assert_eq!(b.lines, vec!["ligne une", "ligne deux"]);
        assert!(!b.dirty);

        b.lines[1].push('!');
        b.dirty = true;
        b.save().unwrap();
        assert!(!b.dirty);

        let reloaded = fs::read_to_string(&path).unwrap();
        assert_eq!(reloaded, "ligne une\nligne deux!\n");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_errors() {
        let missing = env::temp_dir().join("devx-tui-definitely-missing.txt");
        assert!(Buffer::load(missing).is_err());
    }

    #[test]
    fn save_without_path_errors() {
        let mut b = Buffer::empty();
        b.dirty = true;
        assert!(b.save().is_err());
    }
}
