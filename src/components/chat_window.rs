use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct ChatWindow {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl ChatWindow {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for ChatWindow {
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
            .title("Chat Window")
            .title_bottom("Press 'q' to quit")
            .border_style(Style::default().fg(Color::White));
        frame.render_widget(block, area);
        Ok(())
    }
}
