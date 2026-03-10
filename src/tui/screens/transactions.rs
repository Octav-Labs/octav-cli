use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme;

pub fn render(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from("Date"),
        Cell::from("Type"),
        Cell::from("Chain"),
        Cell::from("Protocol"),
        Cell::from("Assets In"),
        Cell::from("Assets Out"),
        Cell::from("Fees"),
    ])
    .style(theme::header_style())
    .height(1);

    let rows: Vec<Row> = app
        .transactions
        .iter()
        .map(|tx| {
            let type_style = match tx.tx_type.to_lowercase().as_str() {
                "swap" | "trade" => Style::default().fg(Color::Cyan),
                "send" | "transfer" => Style::default().fg(Color::Yellow),
                "receive" => Style::default().fg(Color::Green),
                "stake" | "deposit" | "addliquidity" => Style::default().fg(Color::Magenta),
                "claim" | "harvest" | "removeliquidity" => Style::default().fg(Color::Green),
                "approval" | "approve" => Style::default().fg(theme::MUTED),
                _ => Style::default().fg(Color::White),
            };

            Row::new(vec![
                Cell::from(Span::raw(&tx.date_display)),
                Cell::from(Span::styled(capitalize(&tx.tx_type), type_style)),
                Cell::from(Span::styled(&*tx.chain_name, theme::muted_style())),
                Cell::from(Span::styled(&*tx.protocol_name, theme::muted_style())),
                Cell::from(format_asset_summary(&tx.assets_in)),
                Cell::from(format_asset_summary(&tx.assets_out)),
                Cell::from(if tx.fees_fiat > 0.0 {
                    theme::format_usd(tx.fees_fiat)
                } else {
                    "\u{2014}".to_string()
                }),
            ])
        })
        .collect();

    let title = if app.transactions_total > 0 {
        format!(
            " Transactions ({} loaded of {}) ",
            app.transactions.len(),
            app.transactions_total
        )
    } else {
        format!(" Transactions ({}) ", app.transactions.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(12),
            Constraint::Min(16),
            Constraint::Min(16),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED))
            .title(Span::styled(title, theme::header_style())),
    )
    .row_highlight_style(theme::selected_row());

    frame.render_stateful_widget(table, area, &mut app.transactions_table);
}

fn format_asset_summary(assets: &[crate::tui::data::TxAsset]) -> String {
    if assets.is_empty() {
        return "\u{2014}".to_string();
    }

    let first = &assets[0];
    let text = format!("{} {}", theme::format_balance(first.balance), first.symbol);

    if assets.len() > 1 {
        format!("{} +{} more", text, assets.len() - 1)
    } else {
        text
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
    }
}
