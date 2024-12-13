use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{AppState, Chatter, CurrentScreen};
use crate::AVAILABLE_MODELS;

pub fn ui(frame: &mut Frame, app: &mut AppState) {
    match app.current_screen {
        CurrentScreen::Chat => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let title = Paragraph::new(Text::styled(
                "GPT TUI - Esc to leave, Enter to send",
                Style::default().fg(Color::Green),
            ))
            .block(title_block);

            frame.render_widget(title, chunks[0]);

            let mut lines = Vec::new();
            for message in app.chat_menu.messages.iter().rev() {
                match message.role {
                    Chatter::Human => {
                        lines.push(Line::from(Span::styled(
                            format!("Human : {}", message.message),
                            Style::default().fg(Color::Green),
                        )));
                    }
                    Chatter::AI => {
                        lines.push(Line::from(Span::styled(
                            format!("AI    : {}", message.message),
                            Style::default().fg(Color::Yellow),
                        )));
                    }
                }
            }

            let full_text = Text::from(lines);
            let messages = Paragraph::new(full_text)
                .block(Block::bordered().title("Messages"))
                .wrap(Wrap { trim: false })
                .scroll((app.start_line, 0));

            frame.render_widget(messages, chunks[1]);
            frame.render_widget(&app.chat_menu.text_area, chunks[2]);

            app.used_lines = 10000; // TODO: fix this
        }
        CurrentScreen::MainMenu => {
            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());
            let mode_chosen = AVAILABLE_MODELS[app.selected_mode].as_ref();
            let title = Paragraph::new(Text::styled(
                "Welcome to GPT-TUI \n\n'n' for a new chat \n'q' to quit \n'Tab' to change model \n\nSelected Model: ".to_owned() + mode_chosen,
                Style::default().fg(Color::Green),
            ))
            .block(title_block);

            frame.render_widget(title, frame.area());
        }
    }
}
