use std::{error::Error, io};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;
use tokio;
use crate::{
    app::{AppState, CurrentScreen},
    ui::ui,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = AppState::new();
    app.load_api_key()?;

    enable_raw_mode()?;
    let _ = run_app(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::MainMenu => match key.code {
                    KeyCode::Char('n') => {
                        app.current_screen = CurrentScreen::Chat;
                        if let Err(_) = app.new_chat() {
                            return Ok(false);
                        }
                    }
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    _ => {}
                },
                CurrentScreen::Chat if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Enter => {
                            let res = app.send_message().await;
                            if let Err(_) = res {
                                return Ok(false);
                            }
                        }
                        KeyCode::Backspace => {
                            app.delete_char();
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::MainMenu;
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }
                        KeyCode::Char(value) => {
                            app.enter_char(value);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}