use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{AppState, Chatter, CurrentScreen};

// ANCHOR: method_sig
pub fn ui(frame: &mut Frame, app: &AppState) {
    match app.current_screen {
        CurrentScreen::Chat => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
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
            for message in &app.chat_menu.messages {
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
                .wrap(Wrap { trim: false });
            
            frame.render_widget(messages, chunks[1]);

            let message_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let curr_message = Paragraph::new(Text::styled(
                app.chat_menu.current_inp.clone(),
                Style::default().fg(Color::Green),
            ))
            .block(message_block);

            frame.render_widget(curr_message, chunks[2]);


            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor_position(ratatui::layout::Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                chunks[2].x + app.chat_menu.index as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[2].y + 1,
            ))
        },
        CurrentScreen::MainMenu => {
            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let title = Paragraph::new(Text::styled(
                "Welcome to GPT-TUI \n\n'n' for a new chat \n'q' to quit",
                Style::default().fg(Color::Green),
            ))
            .block(title_block);

            frame.render_widget(title, frame.area());
        },
    }
}