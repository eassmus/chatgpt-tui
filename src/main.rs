use std::env;
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
use crate::{
    app::{AppState, CurrentScreen},
    ui::ui,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let arg = env::args().nth(1);
    let mut app = AppState::new();
    match arg {
        Some(arg) => {
            app.load_api_key(arg.as_str())?;
        }
        _ => {
            println!("Please provide a API Key file path.");
            return Ok(());
        }
    }
    // setup terminal
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    let res = run_app(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<bool, Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        let e = event::read()?;
        if let Event::Key(key) = e {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::MainMenu => match key.code {
                    KeyCode::Char('n') => {
                        app.current_screen = CurrentScreen::Chat;
                        app.new_chat()?;
                    }
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    _ => {}
                },
                CurrentScreen::Chat if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        app.send_message().await?;
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::MainMenu;
                    }
                    KeyCode::Up => {
                        app.move_row_start_up();
                    }
                    KeyCode::Down => {
                        app.move_row_start_down();
                    }
                    _ => {
                        app.enter_char(key.into());
                    }
                },
                _ => {}
            }
        } else if let Event::Mouse(event) = e {
            if app.current_screen == CurrentScreen::Chat {
                match event.kind {
                    event::MouseEventKind::ScrollUp => {
                        app.move_row_start_up();
                    }
                    event::MouseEventKind::ScrollDown => {
                        app.move_row_start_down();
                    }
                    _ => {}
                }
            }
        }
    }
}
