//! Rendu des widgets de la TUI (IDE).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Focus, Mode};

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
    draw_editor(frame, body[1], app);

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
    };
    let (row, col) = app.editor.cursor_position();
    let dirty = if app.editor.dirty { " [*]" } else { "" };
    let path = app
        .editor
        .path
        .as_deref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "—".to_string());
    let specs_count = app.specs.len();

    let line = Line::from(vec![
        Span::styled(" devx ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::styled(
            format!(" [{tag}]"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" {path}{dirty}")),
        Span::styled(format!("  {row}:{col}"), Style::default().fg(Color::Gray)),
        Span::styled(
            format!("  · {specs_count} commandes"),
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn draw_tree(frame: &mut Frame, area: Rect, app: &App) {
    let rows = app.visible();
    let focused = app.focus == Focus::Tree && app.mode == Mode::Normal;
    let inner = area.height.saturating_sub(2) as usize;
    let offset = if rows.len() > inner && inner > 0 {
        app.selection.saturating_sub(inner.saturating_sub(1))
    } else {
        0
    };

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

fn draw_editor(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Editor && app.mode == Mode::Normal;
    if !focused {
        frame.render_widget(
            Paragraph::new(Line::from("Tab pour passer le focus à l'éditeur"))
                .style(Style::default().fg(Color::DarkGray))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Éditeur ")
                        .border_style(Style::default().fg(Color::DarkGray)),
                ),
            area,
        );
        return;
    }

    let editor = &app.editor;
    let row = editor.cursor_row;
    let col = editor.cursor_col;
    let inner_w = area.width.saturating_sub(2) as usize;
    let inner_h = area.height.saturating_sub(2) as usize;
    let gutter_w = 4usize;

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

    let border = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Éditeur ")
        .border_style(border);

    let contents = if inner_h == 0 {
        Vec::new()
    } else {
        build_editor_lines(editor, row, col, y_offset, x_offset, text_w, inner_h)
    };

    frame.render_widget(Paragraph::new(contents).block(block), area);
}

fn build_editor_lines(
    editor: &crate::editor::Buffer,
    row: usize,
    col: usize,
    y_offset: usize,
    x_offset: usize,
    text_w: usize,
    inner_h: usize,
) -> Vec<Line<'static>> {
    let lines = &editor.lines;
    let mut out: Vec<Line> = Vec::with_capacity(inner_h);
    for (idx, text) in lines.iter().enumerate().skip(y_offset).take(inner_h) {
        let is_cursor = idx == row;
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
    let active = app.mode == Mode::Command;
    let cursor = if active { "▏" } else { "" };
    let prefix = if active { ":" } else { " " };
    let content = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Green)),
        Span::styled(prefix, Style::default().fg(Color::Yellow)),
        Span::raw(app.input.clone()),
        Span::styled(cursor, Style::default().fg(Color::Green)),
    ]);
    let border = if active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Commande ")
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
        _ => match app.focus {
            Focus::Tree => {
                "j/k: naviguer · l/Entrée: ouvrir · h: replier · Tab: éditeur · : : commande · Ctrl-P: palette · q: quitter"
                    .to_string()
            }
            Focus::Editor => {
                "flèches: déplacer · Entrée: ligne · BS/Suppr: effacer · Ctrl-S: sauver · Tab: arbre · : : commande"
                    .to_string()
            }
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
