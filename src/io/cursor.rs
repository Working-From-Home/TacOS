/// Cursor module - now wraps display module for backward compatibility
/// All cursor management has been consolidated into display.rs

pub use crate::io::display::{get_pos, set_pos, move_left, move_right, new_line};

