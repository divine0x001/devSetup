//! Rendu des widgets de la TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};

/// Dessine toute l'interface pour une frame.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    draw_status(frame, chunks[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);
    draw_tree(frame, body[0], app);
    draw_output(frame, body[1], app);

    draw_command_bar(frame, chunks[2], app);
    draw_hint(frame, chunks[3], app);

    if app.mode == Mode::Palette {
        draw_palette(frame, area, app);
    }
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Command => "COMMANDE",
        Mode::Palette => "PALETTE",
    };
    let root = app.ctx.workspace.root.display();
    let line = Line::from(vec![
        Span::styled(" devx ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(format!(" {root} ")),
        Span::styled(
            format!("[{mode}]"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn draw_tree(frame: &mut Frame, area: Rect, app: &App) {
    let rows = app.visible();
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
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
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
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(lines).block(block), area);
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
    let cursor = if app.mode == Mode::Command { "▏" } else { "" };
    let content = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Green)),
        Span::raw(app.input.clone()),
        Span::styled(cursor, Style::default().fg(Color::Green)),
    ]);
    let border = if app.mode == Mode::Command {
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
    let text = if app.mode == Mode::Command {
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
    } else {
        "j/k: naviguer · l/Entrée: déplier · h: replier · :/: commande · Ctrl-P: palette · q: quitter"
            .to_string()
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
