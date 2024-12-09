use std::{error::Error, fmt, fs};
use chatgpt::client::ChatGPT;
use chatgpt::prelude::Conversation;

pub enum Chatter {
    AI,
    Human,
}

pub struct ChatMessage {
    pub role: Chatter,
    pub message: String,
}

pub struct ChatState {
    pub messages : Vec<ChatMessage>,
    pub current_inp : String,
    pub index: usize,
}

impl ChatState {
    fn new() -> ChatState {
        ChatState { messages: Vec::new(), current_inp : String::new(), index: 0 }
    }
}

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
            client : None,
            conversation : None,
        }
    }

    pub fn new_chat(&mut self) -> Result<(), Box<dyn Error>> {
        match &self.api_key {
            Some(key) => {
                self.client = Some(ChatGPT::new(key)?);
                self.conversation = Some(self.client.as_mut().unwrap().new_conversation());
                self.chat_menu = ChatState::new();
                Ok(())
            },
            _ => {
                Err(Box::new(KeyError))
            },
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.chat_menu.index.saturating_sub(1);
        self.chat_menu.index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.chat_menu.index.saturating_add(1);
        self.chat_menu.index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.chat_menu.current_inp.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.chat_menu.current_inp
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.chat_menu.index)
            .unwrap_or(self.chat_menu.current_inp.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.chat_menu.index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.chat_menu.index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.chat_menu.current_inp.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.chat_menu.current_inp.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.chat_menu.current_inp = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.chat_menu.current_inp.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.chat_menu.index = 0;
    }

    pub async fn send_message(&mut self) -> Result<(), Box<dyn Error>> {
        self.reset_cursor();
        let message : String = self.chat_menu.current_inp.clone();
        let new_message = ChatMessage {message: self.chat_menu.current_inp.clone(), role: Chatter::Human};
        self.chat_menu.messages.push(new_message);
        self.chat_menu.current_inp = String::new();
        let response = self.conversation.as_mut().unwrap().send_message(&message).await.unwrap();
        let mut message = "".to_owned();
        for choice in response.message_choices {
            message.push_str(&choice.message.content);
        }
        self.chat_menu.messages.push(ChatMessage { role: Chatter::AI, message: message.to_string() });
        Ok(())
    }

    pub fn load_api_key(&mut self) -> Result<(), Box<dyn Error>> {
        let file_path = "/home/pulsar/.config/gpttui/key";
        let contents = fs::read_to_string(file_path)?;
        self.api_key = Some(contents.trim().to_owned());
        Ok(())
    }
}