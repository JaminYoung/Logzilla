use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlashState {
    Idle,
    Connecting,
    Erasing,
    Writing,
    Verifying,
    Done,
    Error(String),
}

impl Default for FlashState {
    fn default() -> Self {
        FlashState::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashProgress {
    pub state: FlashState,
    pub percent: u8,
    pub message: String,
    pub current_op: String,
}

impl Default for FlashProgress {
    fn default() -> Self {
        Self {
            state: FlashState::Idle,
            percent: 0,
            message: String::new(),
            current_op: String::new(),
        }
    }
}

impl FlashProgress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_state(&mut self, state: FlashState, percent: u8, message: &str) {
        self.state = state;
        self.percent = percent;
        self.message = message.to_string();
    }
}
