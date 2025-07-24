use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    SendMessage(String),
    MessageReceived(String),
    FocusInput,
    FocusChat,
    ShowDialog(String),      // Show dialog with content
    HideDialog,              // Hide dialog
    ShowSystemPromptDialog,  // Show system prompt dialog
    SetSystemPrompt(String), // Set the system prompt
}
