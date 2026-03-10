use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui_image::StatefulImage;

use crate::tui::app::App;
use crate::tui::theme;

// 5 lines of content + 1 separator
const CARD_HEIGHT: u16 = 6;
const IMG_WIDTH: u16 = 8;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED))
        .title(Span::styled(
            format!(
                " All Holdings — sorted by value ({}) ",
                app.holdings.len()
            ),
            theme::header_style(),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.holdings.is_empty() {
        return;
    }

    let visible_cards = (inner.height / CARD_HEIGHT) as usize;
    if visible_cards == 0 {
        return;
    }

    let selected = app.holdings_table.selected().unwrap_or(0);

    let scroll_offset = if selected >= visible_cards {
        selected - visible_cards + 1
    } else {
        0
    };

    // Clone holding data to avoid borrow conflict with app.token_images
    let holdings_snapshot: Vec<(String, String, String, String, f64, f64, f64)> = app
        .holdings
        .iter()
        .map(|h| {
            (
                h.symbol.clone(),
                h.chain.clone(),
                h.protocol.clone(),
                h.image_url.clone(),
                h.balance,
                h.price,
                h.value,
            )
        })
        .collect();

    for i in 0..visible_cards {
        let idx = scroll_offset + i;
        if idx >= holdings_snapshot.len() {
            break;
        }

        let (ref symbol, ref chain, ref protocol, ref image_url, balance, price, value) =
            holdings_snapshot[idx];
        let is_selected = idx == selected;

        let card_y = inner.y + (i as u16) * CARD_HEIGHT;
        if card_y + CARD_HEIGHT > inner.y + inner.height {
            break;
        }

        let content_height: u16 = 5;
        let content_area = Rect {
            x: inner.x,
            y: card_y,
            width: inner.width,
            height: content_height,
        };

        // Background for selected card
        if is_selected {
            let bg = Paragraph::new(
                (0..content_height).map(|_| Line::from("")).collect::<Vec<_>>(),
            )
            .style(Style::default().bg(theme::SELECTED_BG));
            frame.render_widget(bg, content_area);
        }

        // Image area
        let img_area = Rect {
            x: content_area.x,
            y: content_area.y,
            width: IMG_WIDTH.min(content_area.width),
            height: content_height,
        };

        // Try to render image
        let has_image = if !image_url.is_empty() {
            if let Some(protocol_state) = app.token_images.get_mut(image_url.as_str()) {
                let img_widget = StatefulImage::new();
                frame.render_stateful_widget(img_widget, img_area, protocol_state);
                true
            } else {
                false
            }
        } else {
            false
        };

        if !has_image {
            let mut lines: Vec<Line> = Vec::new();
            for r in 0..content_height {
                if r == content_height / 2 {
                    lines.push(Line::from(Span::styled(
                        "    \u{25c6}   ",
                        Style::default().fg(theme::PRIMARY),
                    )));
                } else {
                    lines.push(Line::from(""));
                }
            }
            let fallback = Paragraph::new(lines);
            frame.render_widget(fallback, img_area);
        }

        // Text area (2-col gap after image)
        let gap: u16 = 2;
        let text_x = content_area.x + IMG_WIDTH + gap;
        let text_width = content_area.width.saturating_sub(IMG_WIDTH + gap);
        if text_width == 0 {
            continue;
        }

        // Build text lines with vertical padding to center 3 lines in 5 rows
        let value_str = theme::format_usd(value);
        let line1_right = format!("Value: {}", value_str);
        let line1_left_budget =
            text_width.saturating_sub(line1_right.len() as u16 + 2) as usize;
        let sym_chain = format!("{:<8} {}", symbol.to_uppercase(), chain);
        let sym_chain_truncated = if sym_chain.len() > line1_left_budget {
            sym_chain[..line1_left_budget].to_string()
        } else {
            sym_chain.clone()
        };
        let padding1 = text_width as usize
            - sym_chain_truncated.len().min(text_width as usize)
            - line1_right.len().min(text_width as usize);

        let line1 = Line::from(vec![
            Span::styled(
                sym_chain_truncated,
                if is_selected {
                    Style::default().fg(theme::ACCENT)
                } else {
                    Style::default().fg(ratatui::style::Color::White)
                },
            ),
            Span::raw(" ".repeat(padding1)),
            Span::styled(line1_right, theme::value_style()),
        ]);

        let line2 = Line::from(vec![
            Span::styled("Protocol: ", theme::muted_style()),
            Span::raw(protocol.as_str()),
        ]);

        let bal_str = theme::format_balance(balance);
        let price_str = theme::format_usd(price);
        let line3 = Line::from(vec![
            Span::styled("Bal: ", theme::muted_style()),
            Span::raw(bal_str),
            Span::raw("   "),
            Span::styled("Price: ", theme::muted_style()),
            Span::raw(price_str),
        ]);

        // Pad: blank, line1, line2, line3, blank — centered in 5 rows
        let text_paragraph = Paragraph::new(vec![
            Line::from(""),
            line1,
            line2,
            line3,
            Line::from(""),
        ]);
        let text_area = Rect {
            x: text_x,
            y: content_area.y,
            width: text_width,
            height: content_height,
        };
        frame.render_widget(text_paragraph, text_area);

        // Separator line
        let sep_y = card_y + content_height;
        if sep_y < inner.y + inner.height {
            let sep = Paragraph::new(Line::from(Span::styled(
                "\u{2500}".repeat(inner.width as usize),
                theme::muted_style(),
            )));
            let sep_area = Rect {
                x: inner.x,
                y: sep_y,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(sep, sep_area);
        }
    }
}
