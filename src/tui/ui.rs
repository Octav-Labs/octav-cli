use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::tui::app::{App, LoadState, ProtocolLevel, Screen};
use crate::tui::{screens, theme};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Header: logo + address + tabs
            Constraint::Min(1),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_screen(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // ASCII logo
            Constraint::Length(1), // Address line
            Constraint::Length(1), // Blank separator
            Constraint::Length(1), // Tab bar
        ])
        .split(area);

    // --- ASCII logo ---
    let logo_style = Style::default().fg(theme::PRIMARY);
    let logo_lines: Vec<Line> = vec![
        Line::from(Span::styled(" \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2557}", logo_style)),
        Line::from(Span::styled(" \u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\u{255a}\u{2550}\u{2550}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}", logo_style)),
        Line::from(Span::styled(" \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}        \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}", logo_style)),
        Line::from(Span::styled(" \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}        \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2588}\u{2588}\u{2551}\u{255a}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2554}\u{255d}", logo_style)),
        Line::from(Span::styled(" \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}  \u{2588}\u{2588}\u{2551} \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}", logo_style)),
        Line::from(Span::styled("  \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}  \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}   \u{255a}\u{2550}\u{255d}   \u{255a}\u{2550}\u{255d}  \u{255a}\u{2550}\u{255d}  \u{255a}\u{2550}\u{2550}\u{2550}\u{255d}", logo_style)),
    ];
    let logo = Paragraph::new(logo_lines);
    frame.render_widget(logo, header_chunks[0]);

    // Credits overlay on the right side of the logo area (row 3)
    if let Some(credits) = app.credits {
        let credits_text = format!("Credits: {} ", theme::format_number(credits, 0));
        let w = credits_text.len() as u16;
        let credits_area = Rect {
            x: header_chunks[0].x + header_chunks[0].width.saturating_sub(w),
            y: header_chunks[0].y + 2, // 3rd line of logo
            width: w.min(header_chunks[0].width),
            height: 1,
        };
        let widget = Paragraph::new(Span::styled(credits_text, theme::muted_style()));
        frame.render_widget(widget, credits_area);
    }

    // --- Address line ---
    let truncated_addresses: Vec<String> = app
        .addresses
        .iter()
        .map(|a| {
            if a.len() > 10 {
                format!("{}...{}", &a[..6], &a[a.len() - 5..])
            } else {
                a.clone()
            }
        })
        .collect();
    let addr_text = format!(" {}", truncated_addresses.join("  "));
    let addr_line = Paragraph::new(Span::styled(addr_text, theme::muted_style()));
    frame.render_widget(addr_line, header_chunks[1]);

    // [r] Refresh hint on the right of the address line
    let refresh_hint = "[r] Refresh ";
    let rw = refresh_hint.len() as u16;
    let refresh_area = Rect {
        x: header_chunks[1].x + header_chunks[1].width.saturating_sub(rw),
        y: header_chunks[1].y,
        width: rw.min(header_chunks[1].width),
        height: 1,
    };
    let refresh_widget = Paragraph::new(Span::styled(refresh_hint, theme::muted_style()));
    frame.render_widget(refresh_widget, refresh_area);

    // --- Tab bar ---
    let titles: Vec<Line> = Screen::all()
        .iter()
        .map(|s| {
            if *s == app.active_screen {
                Line::from(Span::styled(
                    format!(" {} ", s.label()),
                    theme::tab_active(),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", s.label()),
                    theme::tab_inactive(),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_screen.index())
        .divider(Span::raw("│"));

    frame.render_widget(tabs, header_chunks[3]);
}

fn render_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.load_state {
        LoadState::Idle | LoadState::Loading => {
            let dots = match (app.tick_count / 5) % 4 {
                0 => "   ",
                1 => ".  ",
                2 => ".. ",
                _ => "...",
            };
            let msg = format!(" Loading{}", dots);
            let loading = Paragraph::new(msg)
                .style(Style::default().fg(theme::PRIMARY))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(loading, area);
        }
        LoadState::Error(_) | LoadState::Ready => {
            render_active_screen(frame, app, area);
        }
    }
}

fn render_active_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.active_screen {
        Screen::Overview => screens::overview::render(frame, app, area),
        Screen::Holdings => screens::holdings::render(frame, app, area),
        Screen::Protocols => screens::protocols::render(frame, app, area),
        Screen::Transactions => screens::transactions::render(frame, app, area),
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (keys, context) = match app.active_screen {
        Screen::Overview => (
            "[Tab] Next  [1-4] Jump  [r] Refresh  [q] Quit",
            String::new(),
        ),
        Screen::Holdings => (
            "[j/k] Scroll  [g/G] Top/Bottom  [Tab] Next  [r] Refresh  [q] Quit",
            if !app.holdings.is_empty() {
                let selected = app.holdings_table.selected().unwrap_or(0) + 1;
                format!("{} of {}", selected, app.holdings.len())
            } else {
                String::new()
            },
        ),
        Screen::Protocols => match &app.protocol_level {
            ProtocolLevel::List => (
                "[Enter] Drill in  [j/k] Scroll  [Tab] Next  [r] Refresh  [q] Quit",
                String::new(),
            ),
            ProtocolLevel::Detail { .. } => (
                "[Enter] See assets  [Esc] Back  [j/k] Scroll  [q] Quit",
                String::new(),
            ),
            ProtocolLevel::Assets { .. } => (
                "[Esc] Back  [j/k] Scroll  [q] Quit",
                String::new(),
            ),
        },
        Screen::Transactions => (
            "[j/k] Scroll  [g/G] Top/Bottom  [Tab] Next  [r] Refresh  [q] Quit",
            if !app.transactions.is_empty() {
                let selected = app.transactions_table.selected().unwrap_or(0) + 1;
                format!("{} of {}", selected, app.transactions.len())
            } else {
                String::new()
            },
        ),
    };

    let mut spans = vec![Span::styled(format!(" {} ", keys), theme::muted_style())];

    if !context.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            context,
            Style::default().fg(theme::ACCENT),
        ));
    }

    if let Some(ref err) = app.last_error {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Error: {}", err),
            theme::negative_style(),
        ));
    }

    let status = Paragraph::new(Line::from(spans));
    frame.render_widget(status, area);
}
