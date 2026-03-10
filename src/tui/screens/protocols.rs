use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::{App, ProtocolLevel};
use crate::tui::theme;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.protocol_level.clone() {
        ProtocolLevel::List => render_list(frame, app, area),
        ProtocolLevel::Detail { protocol_index } => {
            render_detail(frame, app, area, protocol_index)
        }
        ProtocolLevel::Assets {
            protocol_index,
            chain_index,
            position_index,
        } => render_assets(frame, app, area, protocol_index, chain_index, position_index),
    }
}

fn render_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("  Protocol"),
        Cell::from("Chains"),
        Cell::from("Positions"),
        Cell::from("Total Value"),
    ])
    .style(theme::header_style())
    .height(1);

    let rows: Vec<Row> = app
        .protocols
        .iter()
        .map(|p| {
            let chain_count = p.chains.len();
            let pos_count: usize = p.chains.iter().map(|c| c.positions.len()).sum();

            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("\u{25c6} ", Style::default().fg(theme::PRIMARY)),
                    Span::raw(&p.name),
                ])),
                Cell::from(chain_count.to_string()),
                Cell::from(pos_count.to_string()),
                Cell::from(Span::styled(
                    theme::format_usd(p.total_value),
                    theme::value_style(),
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED))
            .title(Span::styled(" Protocols ", theme::header_style())),
    )
    .row_highlight_style(theme::selected_row());

    frame.render_stateful_widget(table, area, &mut app.protocol_list_state);
}

fn render_detail(frame: &mut Frame, app: &mut App, area: Rect, protocol_index: usize) {
    let proto = match app.protocols.get(protocol_index) {
        Some(p) => p,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Breadcrumb
            Constraint::Min(1),   // Content
        ])
        .split(area);

    // Breadcrumb
    let breadcrumb = Paragraph::new(Line::from(vec![
        Span::styled(" Protocols", theme::muted_style()),
        Span::styled(" \u{25b8} ", Style::default().fg(theme::MUTED)),
        Span::styled(
            &*proto.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(breadcrumb, chunks[0]);

    // Flat list of positions grouped by chain
    let mut rows: Vec<Row> = Vec::new();

    for chain in &proto.chains {
        for pos in &chain.positions {
            rows.push(Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("\u{25c6} ", Style::default().fg(theme::PRIMARY)),
                    Span::raw(&pos.name),
                ])),
                Cell::from(&*chain.name),
                Cell::from(pos.assets.len().to_string()),
                Cell::from(Span::styled(
                    theme::format_usd(pos.total_value),
                    theme::value_style(),
                )),
            ]));
        }
    }

    let header = Row::new(vec![
        Cell::from("  Position"),
        Cell::from("Chain"),
        Cell::from("Assets"),
        Cell::from("Value"),
    ])
    .style(theme::header_style())
    .height(1);

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED))
            .title(Span::styled(
                format!(
                    " {} \u{2014} {} ",
                    proto.name,
                    theme::format_usd(proto.total_value)
                ),
                theme::header_style(),
            )),
    )
    .row_highlight_style(theme::selected_row());

    frame.render_stateful_widget(table, chunks[1], &mut app.protocol_detail_state);
}

fn render_assets(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    protocol_index: usize,
    chain_index: usize,
    position_index: usize,
) {
    let (proto, chain, pos) = match app
        .protocols
        .get(protocol_index)
        .and_then(|p| p.chains.get(chain_index).map(|c| (p, c)))
        .and_then(|(p, c)| c.positions.get(position_index).map(|pos| (p, c, pos)))
    {
        Some(t) => t,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Breadcrumb
            Constraint::Min(1),   // Content
        ])
        .split(area);

    // Breadcrumb
    let breadcrumb = Paragraph::new(Line::from(vec![
        Span::styled(" Protocols", theme::muted_style()),
        Span::styled(" \u{25b8} ", Style::default().fg(theme::MUTED)),
        Span::styled(&*proto.name, theme::muted_style()),
        Span::styled(" \u{25b8} ", Style::default().fg(theme::MUTED)),
        Span::styled(
            format!("{} ({})", pos.name, chain.name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(breadcrumb, chunks[0]);

    let header = Row::new(vec![
        Cell::from("  Asset"),
        Cell::from("Type"),
        Cell::from("Balance"),
        Cell::from("Price"),
        Cell::from("Value"),
    ])
    .style(theme::header_style())
    .height(1);

    let rows: Vec<Row> = pos
        .assets
        .iter()
        .map(|a| {
            let value_style = if a.value < 0.0 {
                theme::negative_style()
            } else {
                theme::value_style()
            };

            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("\u{25c6} ", Style::default().fg(theme::PRIMARY)),
                    Span::raw(&a.symbol),
                ])),
                Cell::from(if a.asset_type.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    a.asset_type.clone()
                }),
                Cell::from(theme::format_balance(a.balance)),
                Cell::from(theme::format_usd(a.price)),
                Cell::from(Span::styled(theme::format_usd(a.value), value_style)),
            ])
        })
        .collect();

    let net_value: f64 = pos.assets.iter().map(|a| a.value).sum();

    let table = Table::new(
        rows,
        [
            Constraint::Min(12),
            Constraint::Min(10),
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
                    " {} \u{2014} {} ",
                    pos.name,
                    theme::format_usd(net_value)
                ),
                theme::header_style(),
            )),
    )
    .row_highlight_style(theme::selected_row());

    frame.render_stateful_widget(table, chunks[1], &mut app.protocol_assets_state);
}
