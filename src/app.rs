use std::env;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{Component, chat_window::ChatWindow, dialog::Dialog, home::Home, input::Input},
    config::Config,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    state: AppState,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub chat_history: Vec<ChatMessage>,
    pub is_loading: bool,
    pub system_prompt: String,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let state = AppState::default();
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(Home::new()),
                Box::new(ChatWindow::new()),
                Box::new(Input::new()),
                Box::new(Dialog::new()),
            ],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::Home,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_state_handler(self.state.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui).await?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => {
                // First, let components handle the key event
                let mut key_handled = false;
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(event.clone()))? {
                        action_tx.send(action)?;
                        key_handled = true;
                    }
                }

                // Only process global keybindings if no component handled the key
                if !key_handled {
                    self.handle_key_event(key)?;
                }
            }
            _ => {
                // For non-key events, let all components handle them
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(event.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    async fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match &action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, *w, *h)?,
                Action::Render => self.render(tui)?,
                Action::Error(err) => {
                    // Clear loading state on error and show error message
                    self.state.is_loading = false;
                    self.state.chat_history.push(ChatMessage {
                        role: "system".to_string(),
                        content: format!("Error: {err}"),
                    });
                    // Update state in all components
                    for component in self.components.iter_mut() {
                        component.register_state_handler(self.state.clone())?;
                    }
                    self.render(tui)?;
                }
                Action::SendMessage(message) => {
                    self.state.chat_history.push(ChatMessage {
                        role: "user".to_string(),
                        content: message.clone(),
                    });
                    debug!("Message sent: {}", message);

                    // Set loading state
                    self.state.is_loading = true;
                    // Update state in all components
                    for component in self.components.iter_mut() {
                        component.register_state_handler(self.state.clone())?;
                    }
                    // Force immediate render to show loading state
                    self.render(tui)?;

                    // Spawn API call in background to avoid blocking the event loop
                    let action_tx = self.action_tx.clone();
                    let chat_history = self.state.chat_history.clone();
                    let system_prompt = self.state.system_prompt.clone();
                    tokio::spawn(async move {
                        let result = async {
                            let client = reqwest::Client::new();

                            // Prepare messages with optional system prompt
                            let mut messages = Vec::new();

                            // Add system prompt if it exists and is not empty
                            if !system_prompt.is_empty() {
                                messages.push(json!({
                                    "role": "system",
                                    "content": system_prompt
                                }));
                            }

                            // Add chat history
                            messages.extend(chat_history.iter().map(|msg| {
                                json!({
                                    "role": msg.role,
                                    "content": msg.content
                                })
                            }));

                            let response = client
                                .post("https://openrouter.ai/api/v1/chat/completions")
                                .header("Content-Type", "application/json")
                                .bearer_auth(env::var("OPENROUTER_API_KEY").map_err(|_| {
                                    color_eyre::eyre::eyre!(
                                        "OPENROUTER_API_KEY environment variable not set"
                                    )
                                })?)
                                .body(
                                    json!({
                                        "model": "mistralai/mistral-nemo",
                                        "messages": messages
                                    })
                                    .to_string(),
                                )
                                .send()
                                .await?;
                            let response_text = response.text().await?;
                            let response_json: serde_json::Value =
                                serde_json::from_str(&response_text)?;
                            let content = response_json["choices"][0]["message"]["content"]
                                .as_str()
                                .unwrap();
                            Ok::<String, color_eyre::eyre::Error>(content.to_string())
                        }
                        .await;

                        match result {
                            Ok(content) => {
                                let _ = action_tx.send(Action::MessageReceived(content));
                            }
                            Err(err) => {
                                let _ = action_tx.send(Action::Error(format!("API Error: {err}")));
                            }
                        }
                    });
                }
                Action::MessageReceived(content) => {
                    self.state.chat_history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: content.clone(),
                    });

                    // Clear loading state
                    self.state.is_loading = false;
                    // Update state in all components
                    for component in self.components.iter_mut() {
                        component.register_state_handler(self.state.clone())?;
                    }
                    // Force immediate render to show response
                    self.render(tui)?;
                }
                Action::SetSystemPrompt(prompt) => {
                    self.state.system_prompt = prompt.clone();
                    // Update state in all components
                    for component in self.components.iter_mut() {
                        component.register_state_handler(self.state.clone())?;
                    }
                }
                Action::FocusInput | Action::FocusChat => {
                    // Handle focus changes if needed
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            let main_area = frame.area();

            // Create main layout: chat area + input area
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Ratio(3, 4), // Chat area 3/4 of the screen
                    Constraint::Ratio(1, 4), // Input area 1/4 of the screen
                ])
                .split(main_area);

            let chat_area = main_layout[0];
            let input_area = main_layout[1];

            // Render components in their designated areas
            for component in self.components.iter_mut() {
                let result = match component.as_any().type_id() {
                    id if id == std::any::TypeId::of::<ChatWindow>() => {
                        component.draw(frame, chat_area)
                    }
                    id if id == std::any::TypeId::of::<Input>() => {
                        component.draw(frame, input_area)
                    }
                    id if id == std::any::TypeId::of::<Dialog>() => {
                        // Dialog should render over the entire screen
                        component.draw(frame, main_area)
                    }
                    _ => {
                        // Default to main area for unknown components
                        component.draw(frame, main_area)
                    }
                };

                if let Err(err) = result {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {err:?}")));
                }
            }
        })?;
        Ok(())
    }
}
