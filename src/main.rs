// Mini Compressor GUI using ghostscript (not for now...)
// This version is really just to test iced and rust and understand
//  all I need to properly build the app, I know it's quite ugly...

mod utils;

use std::path::PathBuf;
use std::{fmt, fs};

use lucide_icons::LUCIDE_FONT_BYTES;

use iced::Alignment::Center;
use iced::widget::{
    Column, button, column, container, pick_list, progress_bar, row, scrollable, space, text,
};
use iced::{Color, Element, Fill, Padding, Task, Theme};

/// All available quality presets.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Quality {
    Low,    // Screen
    Middle, // Ebook
    Good,   // Printer
    High,   // Prepress
}

impl Quality {
    /// All available quality presets.
    pub const ALL: &'static [Self] = &[Self::Low, Self::Middle, Self::Good, Self::High];

    fn as_gs_pdfsettings(&self) -> String {
        match self {
            Quality::Low => String::from("screen"),
            Quality::Middle => String::from("ebook"),
            Quality::Good => String::from("printer"),
            Quality::High => String::from("prepress"),
        }
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quality::Low => write!(f, "Low"),
            Quality::Middle => write!(f, "Middle"),
            Quality::Good => write!(f, "Good"),
            Quality::High => write!(f, "High"),
        }
    }
}

#[derive(Debug)]
struct App {
    theme: Theme,                 // theme selection
    running: bool,                // with progress bar
    files: Vec<utils::FileEntry>, // the list of file to compress
    output_folder: PathBuf,       // the folder where the compressed file should land...
    quality: Quality,             // a preset that chose the compression percentage
    compress_size: u32,           // a slide chosing the size output
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
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDialog => Task::perform(utils::open_file_selection(), Message::AddFiles),

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
                    Task::perform(
                        utils::compress_pdf(
                            path,
                            0,
                            self.quality.as_gs_pdfsettings(),
                            self.output_folder.clone(),
                        ),
                        Message::FileCompressed,
                    )
                } else {
                    Task::none()
                }
            }

            Message::FileCompressed(index) => {
                // chaining logic
                let output_file = utils::format_output_file(
                    &self.files[index].path,
                    &self.output_folder,
                    &self.quality.as_gs_pdfsettings(),
                );

                let file: Result<fs::File, std::io::Error> = fs::File::open(&output_file);

                self.files[index].compressed_size = match file {
                    Ok(file) => file.metadata().expect("REASON").len(),
                    Err(e) => 0,
                };

                self.files[index].compressed = true;

                let next = index + 1;
                if next < self.files.len() {
                    let path = self.files[next].path.clone();
                    Task::perform(
                        utils::compress_pdf(
                            path,
                            next,
                            self.quality.as_gs_pdfsettings(),
                            self.output_folder.clone(),
                        ),
                        Message::FileCompressed,
                    )
                } else {
                    self.running = false;
                    Task::none()
                }
            }
        }
    }

    fn view_header(&self) -> Element<'_, Message> {
        let app_title = text("DocPress").size(25);
        let theme_list = pick_list(Theme::ALL, Some(&self.theme), Message::SelectTheme);
        let quality_list = pick_list(Quality::ALL, Some(&self.quality), Message::SelectQuality);

        let add_document_btn_filed = row![lucide_icons::iced::icon_plus(), text("Add documents"),].spacing(5);
        let output_folder_btn_field = row![lucide_icons::iced::icon_folder_plus(), text("Select Output Folder"),].spacing(5);

        let mangae_files_btn = if self.running {
            row![button(add_document_btn_filed), button(output_folder_btn_field),]
        } else {
            row![
                button(add_document_btn_filed).on_press(Message::OpenDialog),
                button(output_folder_btn_field).on_press(Message::SelectOutputFolder),
            ]
        }
        .spacing(25);

        column![
            row![app_title, theme_list,].spacing(25).align_y(Center),
            row![quality_list, space().width(Fill), mangae_files_btn,]
                .spacing(25)
                .align_y(Center),
            // TODO add a slider. to select the output size...
            // slider(0..=50, app.compress_size, Message::SetSize),
        ]
        .spacing(25)
        .padding(Padding::ZERO.bottom(25))
        .into()
    }

    fn view_footer(&self, files_compressed: f32) -> Element<'_, Message> {
        let progress_ratio = if self.files.is_empty() {
            0.0
        } else {
            (files_compressed / self.files.len() as f32) * 100.0
        };

        let progress_bar = progress_bar(
            0.0..=100.0,
            if self.files.is_empty() {
                0.0
            } else {
                progress_ratio
            },
        )
        .length(Fill);

        let can_compress = self.running
            || self.files.is_empty()
            || self.files.iter().all(|f| f.compressed)
            || !self.output_folder.is_dir();

        let compress_btn = if can_compress {
            button(row![
                    text("Compress"),
                    lucide_icons::iced::icon_rocket(),
                ].spacing(5)
            )
        } else {
            button(row![
                    text("Compress"),
                    lucide_icons::iced::icon_rocket(),
                ].spacing(5)
            ).on_press(Message::Start)
        };

        row![
            row![progress_bar,].padding(Padding::ZERO.right(25)),
            compress_btn,
        ]
        .align_y(Center)
        .padding(Padding::ZERO.top(25))
        .into()
    }

    // ─── View... less fun ───
    fn view(&self) -> Element<'_, Message> {
        let files_compressed = self.files.iter().filter(|f| f.compressed).count() as f32;

        let vec_files: Vec<Element<'_, Message>> = self
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| file.view(index, self.running))
            .collect();

        let files_list = Column::with_children(vec_files);

        column![
            self.view_header(),
            scrollable(files_list).height(Fill),
            self.view_footer(files_compressed),
        ]
        .padding(25)
        .into()
    }
}

impl utils::FileEntry {
    fn view(&self, index: usize, running: bool) -> Element<'_, Message> {
        let content = container(
            row![
                lucide_icons::iced::icon_file_text(),
                text(self.path.file_name().unwrap().display().to_string()),
                text(utils::format_size(self.size)).style(text::base),
                space().width(Fill),
                if self.compressed {
                    row![
                        text(utils::format_size(self.compressed_size)).style(text::success),
                        button(lucide_icons::iced::icon_check()),
                    ]
                    .align_y(Center)
                    .spacing(25)
                } else {
                    if running {
                        row![button(lucide_icons::iced::icon_x()),].align_y(Center)
                    } else {
                        row![button(lucide_icons::iced::icon_x()).on_press(Message::RemoveFile(index)),].align_y(Center)
                    }
                }
            ]
            .spacing(10)
            .align_y(Center)
            .padding(10),
        )
        .style(|theme| {
            let sec = container::secondary(theme);
            sec // TODO change opacity 
        });

        container(content)
            .padding(Padding::ZERO.top(10).bottom(10))
            .into()
    }
}

fn main() -> iced::Result {
    let settings = iced::Settings {
        // add bundled font to iced
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        ..Default::default()
    };

    // TODO status
    iced::application(App::default, App::update, App::view)
        .theme(|app: &App| app.theme.clone())
        .settings(settings)
        .run()
}
