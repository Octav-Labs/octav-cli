use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // NAV card + chain chart
            Constraint::Length(5), // Protocol summary
            Constraint::Min(5),   // Top holdings
        ])
        .split(area);

    // Top row: NAV card + Chain distribution
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    render_nav_card(frame, app, top_row[0]);
    render_chain_chart(frame, app, top_row[1]);
    render_protocol_summary(frame, app, chunks[1]);
    render_top_holdings(frame, app, chunks[2]);
}

fn render_nav_card(frame: &mut Frame, app: &App, area: Rect) {
    let nav_text = match &app.nav {
        Some(nav) => {
            let formatted = theme::format_usd(nav.nav);
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  NET ASSET VALUE",
                    theme::muted_style(),
                )),
                Line::from(Span::styled(
                    format!("  {} {}", formatted, nav.currency),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!(
                        "  {} address{}",
                        app.addresses.len(),
                        if app.addresses.len() != 1 { "es" } else { "" }
                    ),
                    theme::muted_style(),
                )),
            ]
        }
        None => vec![
            Line::from(""),
            Line::from(Span::styled(
                "  NET ASSET VALUE",
                theme::muted_style(),
            )),
            Line::from(Span::styled("  Loading...", theme::muted_style())),
        ],
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED));

    let paragraph = Paragraph::new(nav_text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_chain_chart(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED))
        .title(Span::styled(
            " Chain Distribution ",
            theme::header_style(),
        ));

    if app.chain_allocations.is_empty() {
        let paragraph = Paragraph::new("  No data")
            .style(theme::muted_style())
            .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_bars = (inner.height as usize).min(app.chain_allocations.len());
    let chains = &app.chain_allocations[..max_bars];

    let bar_width = inner.width.saturating_sub(25) as usize;

    let lines: Vec<Line> = chains
        .iter()
        .map(|c| {
            let label = format!(" {:>10} ", truncate(&c.chain_name, 10));
            let filled = ((c.percentage / 100.0) * bar_width as f64) as usize;
            let bar: String =
                "\u{2588}".repeat(filled) + &"\u{2591}".repeat(bar_width.saturating_sub(filled));
            let pct = format!(" {:>5.1}%", c.percentage);

            Line::from(vec![
                Span::styled(label, Style::default().fg(Color::White)),
                Span::styled(bar, Style::default().fg(chain_color(&c.chain_key))),
                Span::styled(pct, theme::muted_style()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_protocol_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED))
        .title(Span::styled(" Protocols ", theme::header_style()));

    if app.protocols.is_empty() {
        let paragraph = Paragraph::new("  No data")
            .style(theme::muted_style())
            .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let max_per_col = inner.height as usize;

    for (col_idx, col_area) in cols.iter().enumerate() {
        let start = col_idx * max_per_col;
        let end = (start + max_per_col).min(app.protocols.len());

        if start >= app.protocols.len() {
            break;
        }

        let lines: Vec<Line> = app.protocols[start..end]
            .iter()
            .map(|p| {
                Line::from(vec![
                    Span::styled(" \u{25c6} ", Style::default().fg(theme::PRIMARY)),
                    Span::styled(
                        format!("{:<15}", truncate(&p.name, 15)),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(theme::format_usd(p.total_value), theme::value_style()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, *col_area);
    }
}

fn render_top_holdings(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("  Token"),
        Cell::from("Chain"),
        Cell::from("Value"),
        Cell::from("%"),
        Cell::from("Price"),
    ])
    .style(theme::header_style())
    .height(1);

    let max_rows = (area.height as usize)
        .saturating_sub(4)
        .min(app.holdings.len())
        .min(10);

    let rows: Vec<Row> = app
        .holdings
        .iter()
        .take(max_rows)
        .map(|h| {
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("\u{25c6} ", Style::default().fg(theme::PRIMARY)),
                    Span::raw(&h.symbol),
                ])),
                Cell::from(Span::styled(&*h.chain, theme::muted_style())),
                Cell::from(Span::styled(
                    theme::format_usd(h.value),
                    theme::value_style(),
                )),
                Cell::from(Span::styled(
                    theme::format_percentage(h.percentage),
                    theme::muted_style(),
                )),
                Cell::from(Span::styled(
                    theme::format_usd(h.price),
                    theme::muted_style(),
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(14),
            Constraint::Min(8),
            Constraint::Min(14),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED))
            .title(Span::styled(
                " Top Holdings ",
                theme::header_style(),
            )),
    );

    frame.render_widget(table, area);
}

fn chain_color(key: &str) -> Color {
    let k = key.to_lowercase();
    if k.contains("ethereum") || k == "eth" {
        Color::Rgb(98, 126, 234)
    } else if k.contains("solana") || k == "sol" {
        Color::Rgb(153, 69, 255)
    } else if k.contains("arbitrum") || k == "arb" {
        Color::Rgb(40, 160, 240)
    } else if k.contains("base") {
        Color::Rgb(0, 82, 255)
    } else if k.contains("polygon") || k.contains("matic") {
        Color::Rgb(130, 71, 229)
    } else if k.contains("optimism") || k == "op" {
        Color::Rgb(255, 4, 32)
    } else if k.contains("avalanche") || k.contains("avax") {
        Color::Rgb(232, 65, 66)
    } else if k.contains("bsc") || k.contains("bnb") {
        Color::Rgb(243, 186, 47)
    } else {
        theme::PRIMARY
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
