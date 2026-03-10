#![allow(dead_code)]
use ratatui::{
    layout::Constraint,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme;

pub fn render(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from("  Token"),
        Cell::from("Chain"),
        Cell::from("Protocol"),
        Cell::from("Balance"),
        Cell::from("Price"),
        Cell::from("Value"),
    ])
    .style(theme::header_style())
    .height(1);

    let rows: Vec<Row> = app
        .holdings
        .iter()
        .map(|h| {
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("\u{25c6} ", Style::default().fg(theme::PRIMARY)),
                    Span::raw(h.symbol.to_uppercase()),
                ])),
                Cell::from(Span::styled(&*h.chain, theme::muted_style())),
                Cell::from(&*h.protocol),
                Cell::from(theme::format_balance(h.balance)),
                Cell::from(theme::format_usd(h.price)),
                Cell::from(Span::styled(
                    theme::format_usd(h.value),
                    theme::value_style(),
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(12),
            Constraint::Min(12),
            Constraint::Min(12),
            Constraint::Min(14),
            Constraint::Min(14),
            Constraint::Min(14),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED))
            .title(Span::styled(
                format!(
                    " All Holdings — sorted by value ({}) ",
                    app.holdings.len()
                ),
                theme::header_style(),
            )),
    )
    .row_highlight_style(theme::selected_row());

    frame.render_stateful_widget(table, area, &mut app.holdings_table);
}
