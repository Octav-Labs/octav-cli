pub mod app;
pub mod data;
pub mod event;
pub mod fetch;
pub mod images;
pub mod screens;
pub mod theme;
pub mod ui;

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use event::DataEvent;

pub fn run(api_key: &str, addresses: &[String]) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Halfblocks picker — reliable at small sizes across all terminals
    let picker = ratatui_image::picker::Picker::halfblocks();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal, api_key, addresses, picker);

    // Always restore terminal, even on error
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    api_key: &str,
    addresses: &[String],
    picker: ratatui_image::picker::Picker,
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<DataEvent>();

    let mut app = App::new(addresses.to_vec(), picker, tx.clone());

    // Start initial data fetch
    fetch::spawn_fetch(api_key.to_string(), addresses.to_vec(), tx.clone());

    let api_key = api_key.to_string();
    let addrs = addresses.to_vec();

    loop {
        // Process data events (non-blocking)
        while let Ok(data_event) = rx.try_recv() {
            app.handle_data_event(data_event);
        }

        // Render
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        // Poll for input events with 100ms timeout (gives ~10fps for animations)
        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::BackTab => app.prev_screen(),
                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.prev_screen();
                        } else {
                            app.next_screen();
                        }
                    }
                    KeyCode::Char('1') => app.active_screen = app::Screen::Overview,
                    KeyCode::Char('2') => app.active_screen = app::Screen::Holdings,
                    KeyCode::Char('3') => app.active_screen = app::Screen::Protocols,
                    KeyCode::Char('4') => app.active_screen = app::Screen::Transactions,
                    KeyCode::Char('r') => {
                        fetch::spawn_fetch(api_key.clone(), addrs.clone(), tx.clone());
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                    KeyCode::Char('g') => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.jump_bottom();
                        } else {
                            app.jump_top();
                        }
                    }
                    KeyCode::Char('G') => app.jump_bottom(),
                    KeyCode::Enter => app.enter(),
                    KeyCode::Esc | KeyCode::Backspace => app.go_back(),
                    _ => {}
                }
            }
        }

        app.tick_count += 1;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
