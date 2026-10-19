//! Rendu des widgets de la TUI (IDE).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Focus, Mode, View, EDITOR_GUTTER};

/// Dessine toute l'interface pour une frame.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    draw_status(frame, chunks[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Min(24)])
        .split(chunks[1]);
    draw_tree(frame, body[0], app);
    match app.view {
        View::Code => {
            // Barre d'onglets + éditeur dans la colonne centrale.
            let center = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(3)])
                .split(body[1]);
            draw_tab_bar(frame, center[0], app);
            draw_editor(frame, center[1], app);
        }
        View::Git => {
            // En vue git, aucun clic éditeur : on neutralise la zone.
            app.layout.editor = Rect::default();
            draw_git(frame, body[1], app);
        }
    }

    draw_output(frame, chunks[2], app);
    draw_command_bar(frame, chunks[3], app);
    draw_hint(frame, chunks[4], app);

    if app.mode == Mode::Palette {
        draw_palette(frame, area, app);
    }
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let tag = match app.mode {
        Mode::Normal => match app.focus {
            Focus::Tree => "ARBRE",
            Focus::Editor => "EDITEUR",
        },
        Mode::Command => "COMMANDE",
        Mode::Palette => "PALETTE",
        Mode::Search => "RECHERCHE",
    };
    let (row, col) = app.editor().cursor_position();
    let total = app.editor().lines.len();
    let dirty = if app.editor().dirty { " [*]" } else { "" };
    let path = app
        .editor()
        .path
        .as_deref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "—".to_string());
    // Indicateur d'onglet affiché seulement s'il y en a plusieurs.
    let tabs = if app.tab_count() > 1 {
        format!("  · onglet {}/{}", app.active + 1, app.tab_count())
    } else {
        String::new()
    };

    let line = Line::from(vec![
        Span::styled(" devx ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::styled(
            format!(" [{tag}]"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" {path}{dirty}")),
        Span::styled(
            format!("  {row}:{col}/{total}"),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(tabs, Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn draw_tree(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = app.visible();
    let focused = app.focus == Focus::Tree && app.mode == Mode::Normal;
    let inner = area.height.saturating_sub(2) as usize;
    let offset = if rows.len() > inner && inner > 0 {
        app.selection.saturating_sub(inner.saturating_sub(1))
    } else {
        0
    };

    // Mémorise la géométrie pour la souris (clic → sélection).
    app.layout.tree = area;
    app.layout.tree_offset = offset;

    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(inner.max(1))
        .map(|(idx, row)| {
            let indent = "  ".repeat(row.depth);
            let icon = if row.is_dir {
                if row.expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "· "
            };
            let text = format!("{indent}{icon}{}", row.name);
            let style = if idx == app.selection {
                if focused {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                }
            } else if row.is_dir {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            Line::from(Span::styled(text, style))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Fichiers ")
        .border_style(if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Greffon (widgets) dessiné au-dessus de l'éditeur : un item par onglet. La
/// barre défile pour garder l'onglet actif visible quand les onglets débordent.
fn draw_tab_bar(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Editor && app.mode == Mode::Normal;
    app.layout.tabs = area;

    // Libellé et largeur de chaque onglet.
    let labels: Vec<(String, u16)> = app
        .tabs
        .iter()
        .map(|tab| {
            let star = if tab.dirty { "*" } else { "" };
            let label = format!(" {}{star} ", tab.display_name());
            let width = label.chars().count() as u16;
            (label, width)
        })
        .collect();

    // Premier onglet visible : le plus petit `first` tel que l'onglet actif
    // tienne dans la zone (défilement horizontal).
    let mut first = 0usize;
    while first < app.active {
        let used: u16 = labels[first..=app.active].iter().map(|(_, w)| *w).sum();
        if used <= area.width {
            break;
        }
        first += 1;
    }

    // Rect de clic par onglet ; zéro (donc ignoré) pour les onglets masqués.
    app.layout.tab_spans = vec![Rect::new(0, 0, 0, 1); app.tabs.len()];

    let mut spans: Vec<Span> = Vec::new();
    let mut x = area.x;
    for (idx, (label, width)) in labels.iter().enumerate().skip(first) {
        // Toujours rendre le premier onglet visible (il contient l'actif),
        // puis s'arrêter dès qu'un onglet ne tient plus.
        if idx != first && x + width > area.x + area.width {
            break;
        }
        app.layout.tab_spans[idx] = Rect::new(x, area.y, *width, 1);
        let style = if idx == app.active {
            Style::default()
                .fg(Color::Black)
                .bg(if focused {
                    Color::Cyan
                } else {
                    Color::DarkGray
                })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(label.clone(), style));
        x += width;
    }
    // Reste de la ligne en fond neutre pour « effacer » l'ancien rendu.
    if x < area.x + area.width {
        spans.push(Span::styled(
            " ".repeat((area.x + area.width - x) as usize),
            Style::default(),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_editor(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Editor && app.mode == Mode::Normal;
    let inner_w = area.width.saturating_sub(2) as usize;
    let inner_h = area.height.saturating_sub(2) as usize;
    let gutter_w = EDITOR_GUTTER;

    // Lecture de l'état de l'onglet actif (emprunt court).
    let (row, col) = {
        let editor = app.editor();
        (editor.cursor_row, editor.cursor_col)
    };

    // Fenêtres de visibilité (défilement vertical et horizontal).
    let y_offset = if inner_h > 0 && row >= inner_h {
        row - inner_h + 1
    } else {
        0
    };
    let text_w = inner_w.saturating_sub(gutter_w).max(1);
    let x_offset = if text_w > 0 && col >= text_w {
        col - text_w + 1
    } else {
        0
    };

    // Mémorise la géométrie pour la souris (clic → curseur).
    app.layout.editor = area;
    app.layout.editor_y_offset = y_offset;
    app.layout.editor_x_offset = x_offset;

    let border = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Éditeur ")
        .border_style(border);

    let search = if app.mode == Mode::Search && !app.search_query.is_empty() {
        Some(app.search_query.as_str())
    } else {
        None
    };
    let selection = app.editor().selection_range();
    let contents = if inner_h == 0 {
        Vec::new()
    } else {
        build_editor_lines(
            app.editor(),
            row,
            col,
            y_offset,
            x_offset,
            text_w,
            inner_h,
            search,
            selection,
        )
    };

    frame.render_widget(Paragraph::new(contents).block(block), area);
}

/// Vue « git » : rapport (branche, modifications, derniers commits) affiché à la
/// place du code.
fn draw_git(frame: &mut Frame, area: Rect, app: &App) {
    let git = &app.git;
    let title = format!(" Git — {} ", git.branch);
    let mut lines: Vec<Line> = Vec::new();

    if let Some(error) = &git.error {
        lines.push(Line::from(Span::styled(
            format!("  ⚠ {error}"),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  (hors dépôt git ou git indisponible — :code pour revenir)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  branche  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                git.branch.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(""));
        let changes = git.status.len();
        lines.push(Line::from(Span::styled(
            format!("  Modifications ({changes})"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        if changes == 0 {
            lines.push(Line::from(Span::styled(
                "    (arbre de travail propre)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for entry in &git.status {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(status_code(entry), Style::default().fg(status_color(entry))),
                    Span::raw(format!(" {}", status_path(entry))),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  Derniers commits ({})", git.log.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        for commit in &git.log {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(commit.clone(), Style::default().fg(Color::Gray)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Magenta));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

/// Deux premiers caractères d'une ligne `git status --short` (le code d'état).
fn status_code(entry: &str) -> String {
    entry.chars().take(2).collect()
}

/// Chemin affiché d'une ligne `git status --short`.
fn status_path(entry: &str) -> String {
    entry.get(3..).unwrap_or("").to_string()
}

/// Couleur associée au code d'état git.
fn status_color(entry: &str) -> Color {
    let code = status_code(entry);
    if code.starts_with('?') {
        Color::DarkGray
    } else if code.contains('D') {
        Color::Red
    } else if code.contains('M') || code.contains('R') {
        Color::Yellow
    } else if code.contains('A') {
        Color::Green
    } else {
        Color::Gray
    }
}

#[allow(clippy::too_many_arguments)]
fn build_editor_lines(
    editor: &crate::editor::Buffer,
    row: usize,
    col: usize,
    y_offset: usize,
    x_offset: usize,
    text_w: usize,
    inner_h: usize,
    search: Option<&str>,
    selection: Option<((usize, usize), (usize, usize))>,
) -> Vec<Line<'static>> {
    let lines = &editor.lines;
    let match_width = search.map(|q| q.chars().count()).unwrap_or(0);
    let mut out: Vec<Line> = Vec::with_capacity(inner_h);
    for (idx, text) in lines.iter().enumerate().skip(y_offset).take(inner_h) {
        let is_cursor = idx == row;
        // Colonnes couvertes par une occurrence du motif de recherche.
        let matches = match search {
            Some(q) => editor.line_match_starts(idx, q),
            None => Vec::new(),
        };
        let in_match = |i: usize| matches.iter().any(|&s| i >= s && i < s + match_width);
        // Cellule (idx, i) comprise dans la sélection [début, fin[.
        let in_selection = |i: usize| match selection {
            Some(((sr, sc), (er, ec))) => {
                (idx > sr || (idx == sr && i >= sc)) && (idx < er || (idx == er && i < ec))
            }
            None => false,
        };
        let gutter = Span::styled(
            format!("{:>3} │ ", idx + 1),
            Style::default().fg(Color::DarkGray),
        );
        let chars: Vec<char> = text.chars().collect();
        let start = x_offset.min(chars.len());
        let mut spans = vec![gutter];
        for (i, c) in chars.iter().enumerate().skip(start).take(text_w) {
            let owned = c.to_string();
            if is_cursor && i == col {
                spans.push(Span::styled(
                    owned,
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if in_match(i) {
                spans.push(Span::styled(
                    owned,
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if in_selection(i) {
                spans.push(Span::styled(
                    owned,
                    Style::default().bg(Color::Blue).fg(Color::White),
                ));
            } else {
                spans.push(Span::raw(owned));
            }
        }
        // Curseur sur une position vide (fin de ligne).
        if is_cursor && col >= chars.len() && col.saturating_sub(x_offset) < text_w {
            spans.push(Span::styled(
                " ",
                Style::default().bg(Color::Yellow).fg(Color::Black),
            ));
        }
        out.push(Line::from(spans));
    }
    out
}

fn draw_output(frame: &mut Frame, area: Rect, app: &App) {
    let inner = area.height.saturating_sub(2) as usize;
    let start = app.output.len().saturating_sub(inner.max(1));
    let lines: Vec<Line> = app.output[start..]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Sortie ")
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_command_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (active, prefix, body, title) = match app.mode {
        Mode::Command => (true, ":", app.input.clone(), " Commande "),
        Mode::Search => (true, "/", app.search_query.clone(), " Recherche "),
        _ => (false, " ", app.input.clone(), " Commande "),
    };
    let cursor = if active { "▏" } else { "" };
    let content = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Green)),
        Span::styled(prefix, Style::default().fg(Color::Yellow)),
        Span::raw(body),
        Span::styled(cursor, Style::default().fg(Color::Green)),
    ]);
    let border = if active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border);
    frame.render_widget(Paragraph::new(content).block(block), area);
}

fn draw_hint(frame: &mut Frame, area: Rect, app: &App) {
    let text = match app.mode {
        Mode::Command => {
            let names: Vec<&str> = app
                .suggestions()
                .iter()
                .take(6)
                .map(|spec| spec.name.as_str())
                .collect();
            if names.is_empty() {
                "Tab: compléter · Entrée: exécuter · Échap: annuler".to_string()
            } else {
                format!("suggestions: {}", names.join("  "))
            }
        }
        Mode::Search => {
            "tape pour chercher · Entrée: occurrence suivante · Échap: annuler".to_string()
        }
        _ => match app.view {
            View::Git => {
                "vue git · :code: revenir au code · Ctrl-G: basculer · : : commande · q (arbre): quitter"
                    .to_string()
            }
            View::Code => match app.focus {
                Focus::Tree => {
                    "j/k: naviguer · clic: ouvrir · l/Entrée: ouvrir · h: replier · Tab: éditeur · : : commande · :git vue git"
                        .to_string()
                }
                Focus::Editor => {
                    "clic: curseur · Ctrl-S: sauver · Ctrl-Z/Y: annuler/rétablir · Ctrl-←/→: onglets · Tab: arbre · : : commande"
                        .to_string()
                }
            },
        },
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::Gray),
        ))),
        area,
    );
}

fn draw_palette(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(area, 60, 60);
    frame.render_widget(Clear, popup);

    let lines: Vec<Line> = app
        .specs
        .iter()
        .enumerate()
        .map(|(idx, spec)| {
            let style = if idx == app.palette_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(
                format!(" {:<14} {}", spec.name, spec.description),
                style,
            ))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Palette de commandes — Ctrl-P ")
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Executor;
    use devx_core::{CommandSpec, Context, Output};
    use ratatui::{backend::TestBackend, Terminal};

    /// Rend l'UI dans un backend de test et renvoie l'écran sous forme de texte.
    fn render(width: u16, height: u16) -> String {
        let ctx = Context::with_root(".");
        let specs = vec![
            CommandSpec::new("tree", "arborescence", "tree"),
            CommandSpec::new("grep", "chercher", "grep <motif>"),
            CommandSpec::new("git", "git", "git <args>"),
        ];
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, specs, exec, None);
        render_app(&mut app, width, height)
    }

    /// Rend une app existante dans un backend de test et renvoie l'écran texte.
    fn render_app(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..height {
            for x in 0..width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn renders_ide_layout() {
        let screen = render(100, 30);
        assert!(screen.contains("Fichiers"), "arbre manquant:\n{screen}");
        assert!(screen.contains("Éditeur"), "éditeur manquant:\n{screen}");
        assert!(screen.contains("Sortie"), "sortie manquante:\n{screen}");
        assert!(
            screen.contains("Commande"),
            "barre de commande manquante:\n{screen}"
        );
        assert!(
            screen.contains("devx"),
            "barre de statut manquante:\n{screen}"
        );
        assert!(
            screen.contains("1:1/1"),
            "indicateur de position manquant:\n{screen}"
        );
    }

    #[test]
    fn renders_palette_overlay() {
        let ctx = Context::with_root(".");
        let specs = vec![CommandSpec::new("tree", "arborescence", "tree")];
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, specs, exec, None);
        app.mode = Mode::Palette;
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut screen = String::new();
        for y in 0..30 {
            for x in 0..100 {
                screen.push_str(buffer[(x, y)].symbol());
            }
            screen.push('\n');
        }
        assert!(
            screen.contains("Palette de commandes"),
            "palette manquante:\n{screen}"
        );
        assert!(
            screen.contains("tree"),
            "commande absente de la palette:\n{screen}"
        );
    }

    #[test]
    fn renders_git_view_instead_of_editor() {
        let ctx = Context::with_root(".");
        let specs = vec![CommandSpec::new("tree", "arborescence", "tree")];
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, specs, exec, None);
        app.view = View::Git;
        app.git.branch = "main".to_string();
        app.git.status = vec![" M src/main.rs".to_string()];
        app.git.log = vec!["abc1234 init".to_string()];
        let screen = render_app(&mut app, 100, 30);
        assert!(
            screen.contains("Git — main"),
            "en-tête git manquant:\n{screen}"
        );
        assert!(
            screen.contains("main.rs"),
            "fichier modifié manquant:\n{screen}"
        );
        assert!(screen.contains("abc1234"), "commit manquant:\n{screen}");
        assert!(
            !screen.contains("Tab pour passer le focus"),
            "l'éditeur ne doit pas être rendu en vue git"
        );
    }

    #[test]
    fn git_view_disables_editor_clicks() {
        let ctx = Context::with_root(".");
        let specs = vec![CommandSpec::new("tree", "arborescence", "tree")];
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, specs, exec, None);
        app.view = View::Git;
        render_app(&mut app, 100, 30);
        assert_eq!(app.layout.editor, Rect::default());
    }

    /// Deux onglets ouverts, prêt au rendu.
    fn app_with_tabs() -> App {
        use crate::Buffer;
        let ctx = Context::with_root(".");
        let specs = vec![CommandSpec::new("tree", "arborescence", "tree")];
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, specs, exec, None);
        app.tabs.push(Buffer::empty_with_path("notes.txt"));
        app
    }

    #[test]
    fn tab_bar_lists_open_buffers() {
        let mut app = app_with_tabs();
        let screen = render_app(&mut app, 100, 30);
        assert!(
            screen.contains("sans-titre"),
            "premier onglet manquant:\n{screen}"
        );
        assert!(screen.contains("notes.txt"), "onglet manquant:\n{screen}");
    }

    #[test]
    fn tab_bar_click_switches_active_tab() {
        let mut app = app_with_tabs();
        let _screen = render_app(&mut app, 100, 30);
        // Le rendu a rempli la géométrie des onglets.
        let rects = app.layout.tab_spans.clone();
        assert_eq!(rects.len(), 2);
        app.on_mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: rects[1].x + 1,
            row: rects[1].y,
            modifiers: crossterm::event::KeyModifiers::NONE,
        });
        assert_eq!(app.active, 1);
        assert_eq!(
            app.editor().path.as_deref(),
            Some(std::path::Path::new("notes.txt"))
        );
    }

    #[test]
    fn tab_bar_marks_dirty_buffer() {
        let mut app = app_with_tabs();
        app.tabs[1].dirty = true;
        let screen = render_app(&mut app, 100, 30);
        assert!(
            screen.contains("notes.txt*"),
            "indicateur de modification manquant:\n{screen}"
        );
    }

    #[test]
    fn tab_bar_scrolls_to_keep_active_visible() {
        use crate::Buffer;
        let ctx = Context::with_root(".");
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, vec![], exec, None);
        for i in 0..6 {
            app.tabs
                .push(Buffer::empty_with_path(format!("fichier{i}.txt")));
        }
        app.active = app.tabs.len() - 1; // dernier onglet actif

        let mut terminal = Terminal::new(TestBackend::new(30, 10)).unwrap();
        terminal.draw(|f| draw(f, &mut app)).unwrap();

        // L'onglet actif a une zone de clic visible ; le premier est masqué.
        assert!(
            app.layout.tab_spans[app.active].width > 0,
            "l'onglet actif doit rester visible"
        );
        assert_eq!(
            app.layout.tab_spans[0].width, 0,
            "les onglets qui débordent sont masqués"
        );
    }

    #[test]
    fn search_mode_highlights_matches() {
        let ctx = Context::with_root(".");
        let specs = vec![CommandSpec::new("tree", "arborescence", "tree")];
        let exec: Executor = Box::new(|_c, _cmd, _args| Ok(Output::empty()));
        let mut app = App::new(ctx, specs, exec, None);
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["trouve ceci ici".to_string()];
        app.mode = Mode::Search;
        app.search_query = "ceci".to_string();

        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let highlighted =
            (0..30).any(|y| (0..100).any(|x| buffer[(x, y)].style().bg == Some(Color::Blue)));
        assert!(highlighted, "aucune occurrence du motif n'est surlignée");
    }
}
