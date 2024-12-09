use std::{error::Error, fmt, fs};
use std::sync::Mutex;
use std::sync::Arc;
use chatgpt::client::ChatGPT;
use chatgpt::prelude::Conversation;
use chatgpt::types::CompletionResponse;
use tokio::task::{AbortHandle, JoinHandle};

pub struct MainMenuState {
    selected_button : u8
}

pub enum Chatter {
    AI,
    Human,
}

impl Chatter {
    pub fn to_string(&self) -> &str {
        match self {
            Chatter::AI => "AI",
            Chatter::Human => "Human",
        }
    }
}

pub struct ChatMessage {
    pub role: Chatter,
    pub message: String,
}

pub struct ChatState {
    pub messages : Vec<ChatMessage>,
    pub current_inp : String,
}

impl ChatState {
    fn new() -> ChatState {
        ChatState { messages: Vec::new(), current_inp : String::new() }
    }
}

pub enum CurrentScreen {
    MainMenu,
    Chat,
}

pub struct AppState {
    pub main_menu: MainMenuState,
    pub chat_menu: ChatState,
    pub api_key: Option<String>,
    pub current_screen: CurrentScreen,
    client: Option<chatgpt::client::ChatGPT>,
    awaiting_api : Option<JoinHandle<Result<CompletionResponse, chatgpt::err::Error>>>,
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
            main_menu: MainMenuState { selected_button : 0 },
            chat_menu: ChatState::new(),
            api_key: None,
            current_screen: CurrentScreen::MainMenu,
            client : None,
            awaiting_api : None,
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

    pub async fn send_message(&mut self) -> Result<(), Box<dyn Error>> {
        if self.awaiting_api.is_some() {
            return Ok(());
        }
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