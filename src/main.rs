// Mini Compressor GUI using ghostscript (not for now...)
// This version is really just to test iced and rust and understand
//  all I need to properly build the app, I know it's quite ugly...

use std::{fs, fmt};
use std::path::PathBuf;

use iced::widget::{button, column, progress_bar, row, scrollable, space, text, pick_list, Column};
use iced::Alignment::Center;
use iced::{Element, Fill, Padding, Task, Theme};
use rfd::AsyncFileDialog;

#[derive(Debug, Clone)]
struct FileEntry {
    path:PathBuf,
    size:u64,
    compressed:bool,
}

// preset either chose a quality preset or chose a size
#[derive(Debug, Clone, PartialEq)]
enum Quality {
    Low,        // 25%
    Middle,     // 50%
    High,       // 75%
}

impl Quality {
    /// A list with all the defined themes.
    pub const ALL: &'static [Self] = &[
        Self::Low,
        Self::Middle,
        Self::High,
    ];
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quality::Low => write!(f, "Low"),
            Quality::Middle => write!(f, "Middle"),
            Quality::High => write!(f, "High"),
        }
    }
}

#[derive(Debug)]
struct App {
    theme:Theme,                // theme selection 
    running:bool,               // with progress bar
    files:Vec<FileEntry>,       // the list of file to compress
    output_folder:PathBuf,      // the folder where the compressed file should land...
    quality:Quality,            // a preset that chose the compression percentage
    compress_size:u32,          // a slide chosing the size output
}

impl Default for App {
    fn default() -> Self {
        Self { 
            theme: Theme::CatppuccinMocha, 
            running: false, 
            files: Vec::new(), 
            output_folder: PathBuf::new(), 
            quality: Quality::Middle, 
            compress_size: 5,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Start,
    AddFiles(Vec<FileEntry>),
    AddOutputFolder(PathBuf),
    RemoveFile(usize),
    FileCompressed(usize),
    OpenDialog,
    SelectOutputFolder,
    SelectTheme(Theme),
    SelectQuality(Quality),
    SetSize(u32),
}

// ─── Initial Fake compressor worker ───
async fn compress_doc(_path:PathBuf, index:usize) -> usize {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    index
}

async fn select_output_folder() -> PathBuf {
    let output_path: Option<rfd::FileHandle> = AsyncFileDialog::new()
        .set_title("Select Output Folder")
        .pick_folder()
        .await;
    
    match output_path {
        Some(folder) => folder.path().to_path_buf(),
        None => PathBuf::new(), // TODO better handling
    }
}

async fn open_file_selection() -> Vec<FileEntry> {
    let selected_files: Option<Vec<rfd::FileHandle>> = AsyncFileDialog::new()
        .add_filter("Pdf", &["pdf"])
        .add_filter("All", &["*"])
        .set_title("Select documents to compress")
        .pick_files()
        .await;

    // Formatting into struct...
    selected_files.iter()
        .flatten()
        .map(|files_handle| {
            let path = files_handle
                                .path()
                                .to_path_buf(); 
            let size = fs::metadata(&path)
                                .map(|m| m.len())
                                .unwrap_or(0);
            FileEntry { path, size, compressed:false }
    }).collect()
}

// ─── Update Mecanic ───
fn update(app: &mut App, message:Message) -> Task<Message> {
    match message {
        Message::OpenDialog => {
            Task::perform(open_file_selection(), Message::AddFiles)
        }

        Message::SelectOutputFolder => {
            Task::perform(select_output_folder(), Message::AddOutputFolder)
        }

        Message::AddFiles(files) => {
            app.files.extend(files);
            Task::none()
        }

        Message::AddOutputFolder(folder) => {
            app.output_folder = folder;
            Task::none()
        }

        Message::RemoveFile(index) => {
            app.files.remove(index);
            Task::none()
        }

        Message::SelectTheme(theme) => {
            app.theme = theme;
            Task::none()
        }

        Message::SelectQuality(quality) => {
            app.quality = quality;
            Task::none()
        }

        Message::SetSize(size) => {
            app.compress_size = size;
            Task::none()
        }

        // TODO use semaphore for Batch task for more performance
        Message::Start => {
            app.running = true;
            if let Some(file) = app.files.first() {
                let path = file.path.clone();
                Task::perform(compress_doc(path, 0), Message::FileCompressed)
            } else {
                Task::none()
            }
        }
        
        Message::FileCompressed(index) => {
            // chaining logic
            app.files[index].compressed = true;
            let next = index + 1;
            if next < app.files.len() {
                let path = app.files[next].path.clone();
                Task::perform(compress_doc(path, next), Message::FileCompressed)
            } else {
                app.running = false;
                Task::none()
            }
        }
    }
}

// ─── View... less fun ───
fn view(app: &App) -> Element<'_, Message> {
    let header = row![
        text("DocPress").size(25),
        space().width(Fill),
        // pick_list(Theme::ALL, Some(&app.theme), Message::SelectTheme),
        pick_list(Quality::ALL, Some(&app.quality), Message::SelectQuality),

        // TODO add a slider. to select the output size...
        // slider(0..=50, app.compress_size, Message::SetSize),

        if app.running {
            button("Add documents")
        } else {
            button("Add documents").on_press(Message::OpenDialog)
        },
        if app.running {
            button("Select Output Folder")
        } else {
            button("Select Output Folder").on_press(Message::SelectOutputFolder)
        },
    ]
    .spacing(25)
    .align_y(Center)
    .padding(Padding::ZERO.bottom(25))
    ;

    let mut files_compressed = 0.0;

    let mut vec_files = Vec::<Element<'_, Message>>::new();
    for (index, file) in app.files.iter().enumerate() {
        vec_files.push(row![
            text(file.path.file_name().unwrap().display().to_string()),
            space().width(Fill),
            if app.files[index].compressed {
                files_compressed += 1.0;
                button("✓")
            } else {
                if app.running {
                    button("❌")
                } else {
                    button("❌").on_press(Message::RemoveFile(index))
                }
            }
        ]
        .spacing(10)
        .padding(Padding::ZERO.top(10).bottom(10))
        .into()
    );
    }

    let files_list = Column::with_children(vec_files);

    let footer: Element<Message> = row![
        row![
            progress_bar(0.0..=100.0, (files_compressed / app.files.len() as f32) * 100.0).length(Fill),
        ]
        .padding(Padding::ZERO.right(25))
        ,
        // space().width(Fill),
        if app.running || app.files.is_empty() || app.files.iter().all(|f| f.compressed) || !app.output_folder.is_dir() {
            button("Compress")
        } else {
            button("Compress").on_press(Message::Start)
        }
    ]
    .align_y(Center)
    .padding(Padding::ZERO.top(25))
    .into();

    column![
        header,
        scrollable(files_list).height(Fill),
        footer,
    ]
    .padding(25)
    .into()
}

fn main() -> iced::Result {
    // TODO theme selector
    // TODO status
    iced::application(App::default, update, view)
        .theme(|app: &App| app.theme.clone())
        .run()

}
