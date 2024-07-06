mod components;
mod editor;
use iced::{Application, Settings};

fn main() -> iced::Result {
    editor::Editor::run(Settings::default())
}
