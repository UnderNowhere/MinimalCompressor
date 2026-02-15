// Mini Compressor GUI using ghostscript (not for now...)
// This version is really just to test iced and rust and understand
//  all I need to properly build the app, I know it's quite ugly...

mod utils;

use std::os::fd::AsFd;
use std::{fmt, fs};
use std::path::PathBuf;

use iced::widget::{button, column, progress_bar, row, scrollable, space, text, pick_list, container, Column};
use iced::Alignment::Center;
use iced::{Element, Fill, Padding, Task, Theme, Color};

/// All available quality presets.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Quality {
    Low,        // Screen
    Middle,     // Ebook
    Good,       // Printer
    High,       // Prepress
}

impl Quality {
    /// All available quality presets.
    pub const ALL: &'static [Self] = &[
        Self::Low,
        Self::Middle,
        Self::Good,
        Self::High,
    ];

    fn as_gs_pdfsettings(&self) -> String {
        match self {
            Quality::Low    => String::from("screen"),
            Quality::Middle => String::from("ebook"),
            Quality::Good   => String::from("printer"),
            Quality::High   => String::from("prepress"),
        }
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quality::Low    => write!(f, "Low"),
            Quality::Middle => write!(f, "Middle"),
            Quality::Good   => write!(f, "Good"),
            Quality::High   => write!(f, "High"),
        }
    }
}

#[derive(Debug)]
struct App {
    theme:Theme,                // theme selection
    running:bool,               // with progress bar
    files:Vec<utils::FileEntry>,// the list of file to compress
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
    AddFiles(Vec<utils::FileEntry>),
    AddOutputFolder(PathBuf),
    RemoveFile(usize),
    FileCompressed(usize),
    OpenDialog,
    SelectOutputFolder,
    SelectTheme(Theme),
    SelectQuality(Quality),
    SetSize(u32),
}

impl App {
    // ─── Update Mecanic ───
    fn update(&mut self, message:Message) -> Task<Message> {
        match message {
            Message::OpenDialog => {
                Task::perform(utils::open_file_selection(), Message::AddFiles)
            }

            Message::SelectOutputFolder => {
                Task::perform(utils::select_output_folder(), Message::AddOutputFolder)
            }

            Message::AddFiles(files) => {
                self.files.extend(files);
                Task::none()
            }

            Message::AddOutputFolder(folder) => {
                self.output_folder = folder;
                Task::none()
            }

            Message::RemoveFile(index) => {
                self.files.remove(index);
                Task::none()
            }

            Message::SelectTheme(theme) => {
                self.theme = theme;
                Task::none()
            }

            Message::SelectQuality(quality) => {
                self.quality = quality;
                Task::none()
            }

            Message::SetSize(size) => {
                self.compress_size = size;
                Task::none()
            }

            // TODO use semaphore for Batch task for more performance
            Message::Start => {
                self.running = true;
                if let Some(file) = self.files.first() {
                    let path = file.path.clone();
                    Task::perform(utils::compress_pdf(path,
                                                0,
                                                self.quality.as_gs_pdfsettings(),
                                                self.output_folder.clone()),
                                            Message::FileCompressed)
                } else {
                    Task::none()
                }
            }

            Message::FileCompressed(index) => {
                // chaining logic
                let output_file = utils::format_output_file(
                    &self.files[index].path,
                    &self.output_folder,
                    &self.quality.as_gs_pdfsettings()
                );

                let file: Result<fs::File, std::io::Error> = fs::File::open(&output_file);

                self.files[index].compressed_size = match file {
                    Ok(file) => {file.metadata().expect("REASON").len()}
                    Err(e) => 0
                };
                
                self.files[index].compressed = true;


                let next = index + 1;
                if next < self.files.len() {
                    let path = self.files[next].path.clone();
                    Task::perform(utils::compress_pdf(path,
                                                next,
                                                self.quality.as_gs_pdfsettings(),
                                                self.output_folder.clone()),
                                                Message::FileCompressed)
                } else {
                    self.running = false;
                    Task::none()
                }
            }
        }
    }

    // ─── View... less fun ───
    fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("DocPress").size(25),
            pick_list(Theme::ALL, Some(&self.theme), Message::SelectTheme),
            space().width(Fill),
            pick_list(Quality::ALL, Some(&self.quality), Message::SelectQuality),

            // TODO add a slider. to select the output size...
            // slider(0..=50, app.compress_size, Message::SetSize),

            if self.running {
                button("Add documents")
            } else {
                button("Add documents").on_press(Message::OpenDialog)
            },
            if self.running {
                button("Select Output Folder")
            } else {
                button("Select Output Folder").on_press(Message::SelectOutputFolder)
            },
        ]
        .spacing(25)
        .align_y(Center)
        .padding(Padding::ZERO.bottom(25))
        ;

        let files_compressed = self.files.iter().filter(|f| f.compressed).count() as f32;

        let vec_files: Vec<Element<'_, Message>> = self.files.iter().enumerate()
            .map(|(index, file)| file.view(index, self.running))
            .collect();

        let files_list = Column::with_children(vec_files);

        let footer: Element<Message> = row![
            row![
                progress_bar(0.0..=100.0, if self.files.is_empty() { 0.0 } else { (files_compressed / self.files.len() as f32) * 100.0 }).length(Fill),
            ]
            .padding(Padding::ZERO.right(25))
            ,
            // space().width(Fill),
            if self.running || self.files.is_empty() || self.files.iter().all(|f| f.compressed) || !self.output_folder.is_dir() {
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
}

impl utils::FileEntry {
    fn view(&self, index: usize, running: bool) -> Element<'_, Message> {
        row![
            text(self.path.file_name().unwrap().display().to_string()),
            text(utils::format_size(self.size)).style(text::secondary),
            space().width(Fill),
            
            if self.compressed {
                row![
                    text(utils::format_size(self.compressed_size)).style(text::success),
                    button("✓"),
                ].align_y(Center).spacing(25)
            } else {
                if running {
                    row![button("❌"),].align_y(Center)
                } else {
                    row![button("❌").on_press(Message::RemoveFile(index)),].align_y(Center)
                }
            }
        ]
        .spacing(10)
        .align_y(Center)
        .padding(Padding::ZERO.top(10).bottom(10))
        .into()
    }
}

fn main() -> iced::Result {
    // TODO status
    iced::application(App::default, App::update, App::view)
        .theme(|app: &App| app.theme.clone())
        .run()

}
