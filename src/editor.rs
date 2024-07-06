use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    executor,
    widget::{button, column, container, row, text, text_editor, Row},
    Command, Theme,
};

type FileData = Result<(Arc<String>, PathBuf), io::ErrorKind>;

#[derive(Debug, Clone)]
pub enum EditorMsg {
    Type(iced::widget::text_editor::Action),
    FileOpened(Option<FileData>),
    Select,
    DialogClosed,
}

pub struct Editor {
    text_content: iced::widget::text_editor::Content,
    current_path: PathBuf,
    err: Option<io::ErrorKind>,
}
impl Editor {
    async fn load_file(path: &Path) -> FileData {
        match tokio::fs::read_to_string(path).await {
            Err(e) => Err(e.kind()),
            Ok(str) => Ok((Arc::new(str), PathBuf::from(path))),
        }
    }
    async fn pick_file() -> Option<FileData> {
        let handler = rfd::AsyncFileDialog::new()
            .set_title("Select file")
            .pick_file()
            .await;
        return if let Some(handler) = handler {
            Some(Self::load_file(handler.path()).await)
        } else {
            None
        };
    }
    fn cursor_data(
        &self,
        textsize: u16,
        spacebetween: u16,
    ) -> Row<<Editor as iced::Application>::Message> {
        let (line, col) = self.text_content.cursor_position();
        let lines = self.text_content.line_count() as f32;

        let percentage = (((line + 1) as f32) / lines * 100.0).round();
        iced::widget::row![
            iced::widget::horizontal_space(iced::Length::Fill),
            iced::widget::text(format!("{}:{}", line, col)).size(textsize),
            iced::widget::horizontal_space(spacebetween),
            iced::widget::text(format!("{percentage}%")).size(textsize),
            iced::widget::horizontal_space(spacebetween)
        ]
    }
}

impl iced::Application for Editor {
    type Message = EditorMsg;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                text_content: text_editor::Content::new(),
                err: None,
                current_path: PathBuf::new(),
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        "Test editor".to_string()
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            EditorMsg::Type(action) => {
                self.text_content.edit(action);
                Command::none()
            }
            EditorMsg::DialogClosed => Command::none(),
            EditorMsg::FileOpened(opt) => {
                if let Some(result) = opt {
                    match result {
                        Ok((content, path)) => {
                            self.text_content = text_editor::Content::with(&content);
                            self.current_path = path;
                        }
                        Err(e) => self.err = Some(e),
                    }
                    Command::none()
                } else {
                    println!("Damn you closed the dialog");
                    Command::none()
                }
            }
            EditorMsg::ReqFile => Command::perform(Self::pick_file(), EditorMsg::FileOpened),
            EditorMsg::ReqFolder => {
                Command::perform(Self.request_folder(), EditorMsg::FolderOpened)
            }
        }
    }
    fn view(&self) -> iced::Element<'_, Self::Message> {
        let row = row![
            button("Open file").on_press(EditorMsg::ReqFile),
            button("Open Folder").on_press(EditorMsg::ReqFolder)
        ];
        let editor = text_editor(&self.text_content).on_edit(EditorMsg::Type);
        container(column![row, editor, self.cursor_data(14, 2)]).into()
    }
    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
