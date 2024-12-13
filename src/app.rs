use chatgpt::client::ChatGPT;
use chatgpt::prelude::Conversation;
use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};
use std::{error::Error, fmt, fs};
use tui_textarea::{Input, TextArea};

pub enum Chatter {
    AI,
    Human,
}

pub struct ChatMessage {
    pub role: Chatter,
    pub message: String,
}

pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub text_area: TextArea<'static>,
}

impl ChatState {
    fn new() -> ChatState {
        let mut text_area = TextArea::default();
        text_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default()),
        );
        text_area.set_cursor_line_style(Style::default());
        ChatState {
            messages: Vec::new(),
            text_area: text_area,
        }
    }
}

#[derive(PartialEq)]
pub enum CurrentScreen {
    MainMenu,
    Chat,
}

pub struct AppState {
    pub chat_menu: ChatState,
    pub api_key: Option<String>,
    pub current_screen: CurrentScreen,
    client: Option<chatgpt::client::ChatGPT>,
    conversation: Option<Conversation>,
    pub start_line: u16,
    pub used_lines: u16,
    pub selected_mode: usize,
}

#[derive(Debug)]
struct KeyError;

impl Error for KeyError {}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            chat_menu: ChatState::new(),
            api_key: None,
            current_screen: CurrentScreen::MainMenu,
            client: None,
            conversation: None,
            start_line: 0,
            used_lines: 0,
            selected_mode: 0,
        }
    }

    pub fn new_chat(
        &mut self,
        engine: chatgpt::prelude::ChatGPTEngine,
    ) -> Result<(), Box<dyn Error>> {
        match &self.api_key {
            Some(key) => {
                self.client = Some(ChatGPT::new_with_config(
                    key,
                    chatgpt::prelude::ModelConfigurationBuilder::default()
                        .engine(engine)
                        .build()?,
                )?);
                self.conversation = Some(self.client.as_mut().unwrap().new_conversation());
                self.chat_menu = ChatState::new();
                Ok(())
            }
            _ => Err(Box::new(KeyError)),
        }
    }

    pub fn enter_char(&mut self, input: Input) {
        self.chat_menu.text_area.input(input);
    }

    pub async fn send_message(&mut self) -> Result<(), Box<dyn Error>> {
        self.start_line = 0;
        let curr_lines = self.chat_menu.text_area.lines();
        let message_text = curr_lines.join("\n");
        let message = ChatMessage {
            message: message_text.clone(),
            role: Chatter::Human,
        };
        self.chat_menu.messages.push(message);
        self.chat_menu
            .text_area
            .move_cursor(tui_textarea::CursorMove::Head);
        self.chat_menu.text_area.delete_str(message_text.len());
        let response = self
            .conversation
            .as_mut()
            .unwrap()
            .send_message(&message_text)
            .await
            .unwrap();
        let mut message = "".to_owned();
        for choice in response.message_choices {
            message.push_str(&choice.message.content);
        }
        self.chat_menu.messages.push(ChatMessage {
            role: Chatter::AI,
            message: message.to_string(),
        });
        Ok(())
    }

    pub fn load_api_key(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(file_path)?;
        self.api_key = Some(contents.trim().to_owned());
        Ok(())
    }

    pub fn move_row_start_up(&mut self) {
        self.start_line = self.start_line.saturating_sub(1);
    }

    pub fn move_row_start_down(&mut self) {
        self.start_line = std::cmp::min(
            self.start_line.saturating_add(1),
            self.used_lines.saturating_sub(1),
        );
    }
}
