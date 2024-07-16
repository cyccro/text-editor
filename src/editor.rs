use std::{collections::HashSet, io, path::PathBuf};

use crate::components::{
    file_tree::{FileHelper, FileTree},
    shortcuts::ShortcutManager,
};
use iced::{
    executor,
    keyboard::{self, KeyCode, Modifiers},
    widget::{column, container, horizontal_space, row, text, text_editor, Row},
    Command, Theme,
};

#[derive(Debug, Clone)]
pub enum EditorMsg {
    Type(iced::widget::text_editor::Action),

    TreeOpenFolder(PathBuf),
    TreeCloseFolder(PathBuf),

    RootOpenFolder(Option<PathBuf>),
    RootOpenFile(Option<PathBuf>),

    ReqFolder,
    ReqFile,

    OpenFile(PathBuf),

    FinishOpenFile(Result<String, io::ErrorKind>),

    SaveFile,

    FinishWrite(Result<(), io::ErrorKind>),

    PressKey(KeyCode, Modifiers),
}

pub struct Editor {
    text_content: iced::widget::text_editor::Content,
    tree: Option<FileTree>,
    err: Option<io::ErrorKind>,
    current_path: PathBuf,
    shortcuts: ShortcutManager,
}
impl Editor {
    fn cursor_data(
        &self,
        textsize: u16,
        spacebetween: u16,
    ) -> Row<<Editor as iced::Application>::Message> {
        let cmd_buf = self.shortcuts.current_keys_str();
        let (line, col) = self.text_content.cursor_position();
        let lines = self.text_content.line_count() as f32;
        let percentage = (((line + 1) as f32) / lines * 100.0).round();
        row![
            text(format!("{cmd_buf} | {}", self.current_path.display())).size(textsize),
            horizontal_space(iced::Length::Fill),
            text(format!("{}:{}", line, col)).size(textsize),
            horizontal_space(spacebetween),
            text(format!("{percentage}%")).size(textsize),
            horizontal_space(spacebetween)
        ]
    }
}
impl iced::Application for Editor {
    type Message = EditorMsg;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut shortcuts = ShortcutManager::new();

        //Open folder, save file, and open folder respectively
        shortcuts.register(vec![KeyCode::LControl, KeyCode::O], |_| EditorMsg::ReqFile);
        shortcuts.register(vec![KeyCode::LControl, KeyCode::S], |_| EditorMsg::SaveFile);
        shortcuts.register(vec![KeyCode::LControl, KeyCode::F], |_| {
            EditorMsg::ReqFolder
        });
        (
            Self {
                text_content: text_editor::Content::new(),
                tree: None,
                err: None,
                current_path: PathBuf::new(),
                shortcuts,
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        "Test editor".to_string()
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            EditorMsg::PressKey(key, modifiers) => self.shortcuts.receive(key, modifiers),
            EditorMsg::Type(action) => {
                if !self.shortcuts.accept {
                    self.text_content.edit(action);
                }
                Command::none()
            }

            EditorMsg::ReqFolder => {
                Command::perform(FileHelper::pick_folder(), EditorMsg::RootOpenFolder)
            }
            EditorMsg::RootOpenFolder(opt) => {
                if let Some(buf) = opt {
                    self.tree = Some(FileTree::new(buf, HashSet::new()));
                } else {
                    println!("Voce fechou o dialogo");
                }
                Command::none()
            }
            EditorMsg::TreeOpenFolder(buf) => {
                if let Some(ref mut tree) = self.tree {
                    tree.opened_folders.insert(buf);
                }
                Command::none()
            }
            EditorMsg::TreeCloseFolder(buf) => {
                if let Some(ref mut tree) = self.tree {
                    tree.opened_folders.remove(&buf);
                }
                Command::none()
            }

            EditorMsg::ReqFile => {
                Command::perform(FileHelper::pick_file(), EditorMsg::RootOpenFile)
            }
            EditorMsg::RootOpenFile(buf) => {
                if let Some(buf) = buf {
                    self.current_path = buf.clone();
                    Command::perform(
                        FileHelper::read_file_content(buf),
                        EditorMsg::FinishOpenFile,
                    )
                } else {
                    Command::none()
                }
            }
            EditorMsg::OpenFile(buf) => {
                self.current_path = buf.clone();
                Command::perform(
                    FileHelper::read_file_content(buf),
                    EditorMsg::FinishOpenFile,
                )
            }
            EditorMsg::FinishOpenFile(content) => {
                if let Ok(content) = content {
                    self.text_content = text_editor::Content::with(&*content);
                }
                Command::none()
            }

            EditorMsg::SaveFile => Command::perform(
                FileHelper::save_file(self.current_path.clone(), self.text_content.text()),
                EditorMsg::FinishWrite,
            ),
            EditorMsg::FinishWrite(result) => {
                match result {
                    Ok(_) => {}
                    Err(e) => self.err = Some(e),
                };
                Command::none()
            }
        }
    }
    fn view(&self) -> iced::Element<'_, Self::Message> {
        let editor = if let Some(tree) = &self.tree {
            column![
                self.cursor_data(12, 8),
                row![
                    tree.view(),
                    text_editor(&self.text_content).on_edit(EditorMsg::Type),
                ],
            ]
            .spacing(5)
        } else {
            column![
                self.cursor_data(12, 8),
                text_editor(&self.text_content).on_edit(EditorMsg::Type),
            ]
            .spacing(5)
        };
        container(editor).into()
    }
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        keyboard::on_key_press(|key, modifiers| Some(EditorMsg::PressKey(key, modifiers)))
    }
    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
