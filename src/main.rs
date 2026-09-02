use crate::editor::Editor;

mod buffer;
mod editor;
mod terminal;
mod view;

fn main() {
    Editor::default().run();
}
