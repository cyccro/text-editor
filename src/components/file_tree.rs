use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    widget::{button, scrollable, text, Column},
    Element, Length,
};

use crate::editor::EditorMsg;

pub type FileData = Result<(Arc<String>, PathBuf), io::ErrorKind>;

pub enum FileLeaf {
    Folder(PathBuf),
    File(PathBuf),
}

pub struct FileTree {
    pub path: PathBuf,
    pub branches: Vec<FileLeaf>,
    pub opened_folders: HashSet<PathBuf>,
}
pub struct FileHelper;
impl FileHelper {
    pub async fn save_file(path: PathBuf, content: String) -> Result<(), io::ErrorKind> {
        tokio::fs::write(path, content.as_bytes())
            .await
            .map_err(|e| e.kind())
    }
    pub fn contents_in_folder_sync(
        path: std::path::PathBuf,
    ) -> Result<Vec<(String, bool)>, std::io::ErrorKind> {
        match std::fs::read_dir(path) {
            Err(e) => Err(e.kind()),
            Ok(mut dir) => {
                let mut vec: Vec<(String, bool)> = Vec::with_capacity(64);
                while let Some(Ok(entry)) = dir.next() {
                    let mut path = format!("{:?}", entry.path())
                        .split('/')
                        .last()
                        .unwrap_or("Err")
                        .to_string();
                    path.pop();
                    vec.push((path, false));
                }
                Ok(vec)
            }
        }
    }
    pub async fn contents_in_folder(
        path: std::path::PathBuf,
    ) -> Result<Vec<(String, bool)>, std::io::ErrorKind> {
        match tokio::fs::read_dir(path).await {
            Err(e) => Err(e.kind()),
            Ok(mut dir) => {
                let mut vec: Vec<(String, bool)> = Vec::with_capacity(64);
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let mut path = format!("{}", entry.path().display())
                        .split('/')
                        .last()
                        .unwrap()
                        .to_string();
                    path.pop();
                    vec.push((path, false));
                }
                Ok(vec)
            }
        }
    }
    async fn load_file(path: &Path) -> FileData {
        match tokio::fs::read_to_string(path).await {
            Err(e) => Err(e.kind()),
            Ok(str) => Ok((Arc::new(str), PathBuf::from(path))),
        }
    }
    pub async fn pick_file_content() -> Option<FileData> {
        let handler = rfd::AsyncFileDialog::new()
            .set_title("Select file")
            .pick_file()
            .await;
        if let Some(handler) = handler {
            Some(Self::load_file(handler.path()).await)
        } else {
            None
        }
    }
    pub async fn pick_file() -> Option<PathBuf> {
        if let Some(data) = rfd::AsyncFileDialog::new()
            .set_title("Select file")
            .pick_file()
            .await
        {
            Some(data.path().to_path_buf())
        } else {
            None
        }
    }
    pub async fn pick_folder() -> Option<PathBuf> {
        match rfd::AsyncFileDialog::new().pick_folder().await {
            Some(handler) => Some(PathBuf::from(handler.path())),
            None => None,
        }
    }
    pub async fn read_file_content(path: PathBuf) -> Result<String, io::ErrorKind> {
        tokio::fs::read_to_string(path).await.map_err(|e| e.kind())
    }
}

impl FileTree {
    pub fn new(path: PathBuf, opened_folders: HashSet<PathBuf>) -> Self {
        Self {
            opened_folders,
            path: path.clone(),
            branches: {
                let mut vec: Vec<FileLeaf> = Vec::new();
                if let Ok(mut dir) = std::fs::read_dir(path) {
                    while let Some(Ok(entry)) = dir.next() {
                        let path = entry.path();
                        vec.push({
                            if path.is_dir() {
                                FileLeaf::Folder(path)
                            } else {
                                FileLeaf::File(path)
                            }
                        });
                    }
                }
                vec
            },
        }
    }
    pub fn file_btn<'a>(buf: PathBuf, level: u16) -> Element<'a, EditorMsg> {
        button(text(format!("{}", buf.display()).split('/').last().unwrap()).size(11))
            .on_press(EditorMsg::OpenFile(buf))
            .padding([0, 0, 0, level * 8])
            .style(iced::theme::Button::Text)
            .into()
    }
    pub fn folder_btn_open<'a>(buf: PathBuf, level: u16) -> Element<'a, EditorMsg> {
        button(text(format!("{}", buf.display()).split('/').last().unwrap()).size(11))
            .on_press(EditorMsg::TreeOpenFolder(buf))
            .padding([0, 0, 0, level * 8])
            .style(iced::theme::Button::Text)
            .into()
    }
    pub fn folder_btn_del<'a>(buf: PathBuf, level: u16) -> Element<'a, EditorMsg> {
        button(text(format!("{}", buf.display()).split('/').last().unwrap()).size(11))
            .on_press(EditorMsg::TreeCloseFolder(buf))
            .padding([0, 0, 0, level * 8])
            .style(iced::theme::Button::Text)
            .into()
    }
    /**
     * Returns a vector with buttons for opening folders and files
     */
    pub fn view_elements<'a>(&self, level: u16) -> Vec<Element<'a, EditorMsg>> {
        let mut elements: Vec<Element<'a, EditorMsg>> = Vec::new();
        for leaf in self.branches.iter() {
            match leaf {
                FileLeaf::Folder(ref path) => {
                    //if the opened folders contains this path, then it gotta push first the contents inside this folder, and then the next ones
                    if self.opened_folders.contains(path) {
                        //push button to set flag false
                        elements.push(Self::folder_btn_del(path.clone(), level));
                        //recurse inside this path
                        let tree = FileTree::new(path.to_path_buf(), self.opened_folders.clone());
                        //add the results in the first elements vector
                        for element in tree.view_elements(level + 1) {
                            elements.push(element);
                        }
                    } else {
                        elements.push(Self::folder_btn_open(path.to_path_buf(), level));
                    };
                }
                FileLeaf::File(path) => elements.push(Self::file_btn(path.to_path_buf(), level)),
            }
        }
        elements
    }
    pub fn view<'a>(&self) -> Element<'a, EditorMsg> {
        scrollable(Column::with_children(self.view_elements(1)))
            .width(Length::Fixed(125.))
            .direction(scrollable::Direction::Vertical(
                scrollable::Properties::new()
                    .width(4)
                    .scroller_width(3)
                    .alignment(scrollable::Alignment::End),
            ))
            .into()
    }
}
