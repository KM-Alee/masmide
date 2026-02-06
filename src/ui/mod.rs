pub mod autocomplete;
pub mod command_bar;
pub mod editor;
pub mod editor_render;
pub mod file_tree;
pub mod help;
pub mod hover;
pub mod input_popup;
pub mod layout;
pub mod output;
pub mod search_bar;
pub mod status_bar;
pub mod tabs;

use crate::app::App;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &mut App) {
    layout::render(frame, app);
}
