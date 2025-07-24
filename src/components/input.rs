use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::Block};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use super::Component;
use crate::{action::Action, config::Config};

pub struct Input {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    textarea: TextArea<'static>,
    is_focused: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        let textarea = TextArea::default();

        Self {
            command_tx: None,
            config: Config::default(),
            textarea,
            is_focused: true,
        }
    }

    #[allow(dead_code)]
    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    #[allow(dead_code)]
    pub fn get_text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
    }

    #[allow(dead_code)]
    fn submit(&mut self) -> Option<Action> {
        let text = self.get_text();
        if !text.trim().is_empty() {
            self.clear();
            Some(Action::SendMessage(text))
        } else {
            None
        }
    }
}

impl Component for Input {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if !self.is_focused {
            return Ok(None);
        }

        match key.code {
            KeyCode::Enter => {
                let text = self.get_text();
                if !text.trim().is_empty() {
                    self.clear();
                    Ok(Some(Action::SendMessage(text)))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Esc => {
                // Clear input on Escape
                self.clear();
                Ok(None)
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                // Ctrl+C to quit
                Ok(Some(Action::Quit))
            }
            _ => {
                {
                    // Let tui-textarea handle all other key events
                    self.textarea.input(key);
                    Ok(None)
                }
            }
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::bordered()
            .title("Input")
            .title_bottom("Esc: clear | Ctrl+C: quit | Use arrow keys, Page Up/Down to navigate")
            .border_style(Style::default().fg(Color::Blue));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(&self.textarea, inner_area);
        Ok(())
    }
}
