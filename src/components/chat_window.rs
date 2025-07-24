use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, app::AppState, config::Config};

#[derive(Default)]
pub struct ChatWindow {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    state: Option<AppState>,
    scroll_offset: usize, // Add scroll offset for navigation
}

impl ChatWindow {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            state: None,
            scroll_offset: 0,
        }
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

    fn register_state_handler(&mut self, state: AppState) -> Result<()> {
        self.state = Some(state);
        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                Ok(None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset += 1;
                Ok(None)
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                Ok(None)
            }
            KeyCode::PageDown => {
                self.scroll_offset += 10;
                Ok(None)
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
                Ok(None)
            }
            KeyCode::End => {
                // Will be handled in draw() to scroll to bottom
                self.scroll_offset = usize::MAX;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // Request render on every tick when loading to animate spinner
                if let Some(ref state) = self.state
                    && state.is_loading
                {
                    return Ok(Some(Action::Render));
                }
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
            .title_bottom("↑↓: scroll | PgUp/PgDn: fast scroll | Home/End: top/bottom")
            .border_style(Style::default().fg(Color::White));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if let Some(ref state) = self.state {
            // Calculate wrapped text for all messages
            let mut wrapped_messages = Vec::new();
            let available_width = inner_area.width.saturating_sub(2) as usize; // Account for padding

            for msg in &state.chat_history {
                let style = if msg.role == "user" {
                    Style::default().fg(Color::White).bg(Color::Black)
                } else {
                    Style::default().fg(Color::Black).bg(Color::Blue)
                };

                // Create role prefix
                let role_prefix = format!("{}: ", msg.role);
                let prefix_len = role_prefix.len();

                // Wrap the content text
                let wrapped_lines =
                    wrap_text(&msg.content, available_width.saturating_sub(prefix_len));

                // First line includes the role prefix
                if let Some(first_line) = wrapped_lines.first() {
                    wrapped_messages.push((format!("{role_prefix}{first_line}"), style));

                    // Subsequent lines are indented
                    for line in wrapped_lines.iter().skip(1) {
                        let indent = " ".repeat(prefix_len);
                        wrapped_messages.push((format!("{indent}{line}"), style));
                    }
                }
            }

            // Add loading indicator if loading
            if state.is_loading {
                let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let spinner_index = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
                    / 100)
                    % spinner_chars.len() as u128;
                let spinner_char = spinner_chars[spinner_index as usize];

                wrapped_messages.push((
                    format!("assistant: {spinner_char} Thinking..."),
                    Style::default().fg(Color::Yellow),
                ));
            }

            // Convert to ListItems
            let items: Vec<ListItem> = wrapped_messages
                .iter()
                .map(|(text, style)| ListItem::new(Text::from(text.clone()).style(*style)))
                .collect();

            // Handle scrolling
            let total_items = items.len();
            let visible_lines = inner_area.height as usize;

            let mut list_state = ListState::default();

            // Clamp scroll offset to valid range
            let max_scroll = total_items.saturating_sub(visible_lines);
            if self.scroll_offset == usize::MAX {
                // End key was pressed - scroll to bottom
                self.scroll_offset = max_scroll;
            } else {
                self.scroll_offset = self.scroll_offset.min(max_scroll);
            }

            if total_items > 0 {
                let selected_index = if total_items <= visible_lines {
                    // All items fit, no scrolling needed
                    None
                } else {
                    // Set selection to control what's visible
                    Some(self.scroll_offset + visible_lines.saturating_sub(1))
                };
                list_state.select(selected_index);
            }

            let chat_history_widget = List::new(items).style(Style::default());

            frame.render_stateful_widget(chat_history_widget, inner_area, &mut list_state);
        }

        Ok(())
    }
}

// Helper function to wrap text to fit within the specified width
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    // Use textwrap for better word wrapping
    let options = textwrap::Options::new(max_width)
        .break_words(true)
        .word_separator(textwrap::WordSeparator::AsciiSpace);

    let wrapped = textwrap::wrap(text, &options);

    if wrapped.is_empty() {
        vec![String::new()]
    } else {
        wrapped.into_iter().map(|cow| cow.into_owned()).collect()
    }
}
