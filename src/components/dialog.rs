use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use super::Component;
use crate::{action::Action, app::AppState, config::Config};

#[derive(Default)]
pub struct Dialog {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    state: Option<AppState>,
    textarea: TextArea<'static>,
    is_visible: bool,
    is_focused: bool,
    dialog_type: DialogType,
}

#[derive(Default, Clone, PartialEq)]
enum DialogType {
    #[default]
    SystemPrompt,
    Generic,
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            state: None,
            textarea: TextArea::default(),
            is_visible: false,
            is_focused: true, // Default to focused when created
            dialog_type: DialogType::default(),
        }
    }

    pub fn show(&mut self, content: String) {
        self.textarea = TextArea::default();
        if !content.is_empty() {
            self.textarea.insert_str(content);
        }
        self.is_visible = true;
        self.is_focused = true; // Focus when showing
        self.dialog_type = DialogType::Generic;
    }

    pub fn show_system_prompt(&mut self, content: String) {
        self.textarea = TextArea::default();
        if !content.is_empty() {
            self.textarea.insert_str(content);
        }
        self.is_visible = true;
        self.is_focused = true; // Focus when showing
        self.dialog_type = DialogType::SystemPrompt;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.is_focused = false; // Unfocus when hiding
        self.textarea = TextArea::default();
    }

    pub fn get_text(&self) -> String {
        self.textarea.lines().join("\n")
    }
}

impl Component for Dialog {
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

    fn register_state_handler(&mut self, state: AppState) -> Result<()> {
        self.state = Some(state);
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // Only handle events when dialog is visible and focused
        if !self.is_visible || !self.is_focused {
            return Ok(None);
        }

        match key.code {
            KeyCode::Esc => Ok(Some(Action::HideDialog)),

            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                // Ctrl+S to submit
                let text = self.get_text();
                let action_to_send = match self.dialog_type {
                    DialogType::SystemPrompt => {
                        if let Some(tx) = &self.command_tx {
                            // Send the system prompt action separately
                            let _ = tx.send(Action::SetSystemPrompt(text));
                        }
                        Action::HideDialog
                    }
                    DialogType::Generic => {
                        // For generic dialogs, just hide
                        Action::HideDialog
                    }
                };
                Ok(Some(action_to_send))
            }
            _ => {
                // Let tui-textarea handle all other key events
                self.textarea.input(key);
                Ok(None)
            }
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ShowDialog(content) => {
                self.show(content);
                // When dialog is shown, it should take focus and input should lose focus
                Ok(Some(Action::Render))
            }
            Action::ShowSystemPromptDialog => {
                // Get current system prompt from state if available
                let current_prompt = if let Some(state) = &self.state {
                    state.system_prompt.clone()
                } else {
                    String::new()
                };
                self.show_system_prompt(current_prompt);
                // When dialog is shown, it should take focus and input should lose focus
                Ok(Some(Action::Render))
            }
            Action::HideDialog => {
                self.hide();
                // When dialog is hidden, input should regain focus
                Ok(Some(Action::FocusInput))
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.is_visible {
            return Ok(());
        }

        // Create a centered dialog area (larger for text editing)
        let dialog_width = area.width.min(80);
        let dialog_height = area.height.min(30);

        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear the background (create overlay effect)
        let clear = Clear;
        frame.render_widget(clear, dialog_area);

        // Create the dialog block with appropriate title and instructions
        let (title, bottom_title) = match self.dialog_type {
            DialogType::SystemPrompt => (
                "System Prompt Editor",
                "Ctrl+Enter or Ctrl+S: Save | Esc: Cancel",
            ),
            DialogType::Generic => ("Text Editor", "Ctrl+Enter or Ctrl+S: Submit | Esc: Cancel"),
        };

        // Set border color based on focus state
        let border_color = if self.is_focused {
            Color::Blue
        } else {
            Color::Gray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black))
            .title(title)
            .title_bottom(bottom_title);

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Render the textarea
        frame.render_widget(&self.textarea, inner_area);

        Ok(())
    }
}
