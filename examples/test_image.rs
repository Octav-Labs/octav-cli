use std::io::{self, Cursor};

use crossterm::{
    event::{poll, read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use ratatui_image::{picker::Picker, StatefulImage};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    eprintln!("Downloading test image...");
    let url = "https://images.octav.fi/tokens/small/usdc_logo_02ef93c7-4314-42cc-9dec-5ba4413026d8.png";

    let client = reqwest::blocking::Client::builder()
        .user_agent("octav-cli/0.1")
        .build()?;
    let resp = client.get(url).send()?;
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let bytes = resp.bytes()?;
    eprintln!("Downloaded {} bytes, content-type: {}", bytes.len(), content_type);

    let dyn_img = image::ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()?
        .decode()?;
    eprintln!("Image decoded: {}x{}", dyn_img.width(), dyn_img.height());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let halfblocks_picker = Picker::halfblocks();
    let query_picker = Picker::from_query_stdio().ok();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut hb_protocol = halfblocks_picker.new_resize_protocol(dyn_img.clone());
    let mut query_protocol = query_picker
        .as_ref()
        .map(|p| p.new_resize_protocol(dyn_img.clone()));

    let query_proto_type = query_picker
        .as_ref()
        .map(|p| format!("{:?}", p.protocol_type()))
        .unwrap_or_else(|| "FAILED".to_string());

    let mut use_halfblocks = true;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let title = format!(
                " [h] Halfblocks  [i] {} | Current: {} | [q] Quit ",
                query_proto_type,
                if use_halfblocks { "Halfblocks" } else { &query_proto_type }
            );

            let block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let sizes: [(u16, u16); 4] = [(6, 3), (10, 5), (14, 7), (20, 10)];
            let mut x_offset = 1u16;

            for (w, h) in sizes {
                if x_offset + w + 2 > inner.width {
                    break;
                }

                let img_area = Rect {
                    x: inner.x + x_offset,
                    y: inner.y + 1,
                    width: w,
                    height: h,
                };

                if use_halfblocks {
                    let widget = StatefulImage::new();
                    frame.render_stateful_widget(widget, img_area, &mut hb_protocol);
                } else if let Some(ref mut proto) = query_protocol {
                    let widget = StatefulImage::new();
                    frame.render_stateful_widget(widget, img_area, proto);
                }

                let label_y = inner.y + 1 + h;
                if label_y < inner.y + inner.height {
                    let label = Paragraph::new(Line::from(Span::styled(
                        format!("{}x{}", w, h),
                        Style::default().fg(Color::Yellow),
                    )));
                    frame.render_widget(label, Rect { x: inner.x + x_offset, y: label_y, width: w, height: 1 });
                }

                x_offset += w + 3;
            }
        })?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => use_halfblocks = true,
                    KeyCode::Char('i') => use_halfblocks = false,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
