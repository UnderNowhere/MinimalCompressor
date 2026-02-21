// Mini Compressor GUI using ghostscript (not for now...)
// This version is really just to test iced and rust and understand
//  all I need to properly build the app, I know it's quite ugly...

mod quality;
mod file_entry;
mod compression;
mod dialog;

use std::path::PathBuf;
use std::fs;

use quality::Quality;

use lucide_icons::LUCIDE_FONT_BYTES;

use iced::Alignment::Center;
use iced::widget::{
    Column, button, column, container, pick_list, progress_bar, row, scrollable, space, text,
};
use iced::{Element, Fill, Padding, Task, Theme};

#[derive(Debug)]
struct App {
    theme: Theme,                 // theme selection
    running: bool,                // with progress bar
    files: Vec<file_entry::FileEntry>, // the list of file to compress
    output_folder: PathBuf,       // the folder where the compressed file should land...
    quality: Quality,             // a preset that chose the compression percentage
    compress_size: u32,           // a slide chosing the size output
    ghostscript_found: bool,      // ghostscript detection at startup
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
            ghostscript_found: compression::is_ghostscript_installed(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Start,
    AddFiles(Vec<file_entry::FileEntry>),
    AddOutputFolder(PathBuf),
    RemoveFile(usize),
    FileCompressed(usize),
    OpenDialog,
    SelectOutputFolder,
    SelectTheme(Theme),
    SelectQuality(quality::Quality),
    SetSize(u32),
    OpenGhostscriptLink,
}

impl App {
    // ─── Update Mecanic ───
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDialog => Task::perform(dialog::open_file_selection(), Message::AddFiles),

            Message::SelectOutputFolder => {
                Task::perform(dialog::select_output_folder(), Message::AddOutputFolder)
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

            Message::OpenGhostscriptLink => {
                let _ = open::that("https://www.ghostscript.com/releases/gsdnld.html");
                Task::none()
            }

            // TODO use semaphore for Batch task for more performance
            Message::Start => {
                self.running = true;
                if let Some(file) = self.files.first() {
                    let path = file.path.clone();
                    Task::perform(
                        compression::compress_pdf(
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
                let output_file = compression::format_output_file(
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
                        compression::compress_pdf(
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
        let presets_txt = text("Presets: ");
        let quality_list = pick_list(quality::Quality::ALL, Some(&self.quality), Message::SelectQuality);

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
            row![presets_txt, quality_list, space().width(Fill), mangae_files_btn,]
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

    // ─── Ghostscript Warning ───
    fn view_ghostscript_warning(&self) -> Element<'_, Message> {
        let warning_icon = lucide_icons::iced::icon_triangle_alert().size(25);
        let warning_text = text("Ghostscript not found! Compression requires Ghostscript.");
        let install_btn = button(
            row![lucide_icons::iced::icon_external_link(), text("Install Ghostscript"),].spacing(5)
            
        ).on_press(Message::OpenGhostscriptLink);

        container(
            row![warning_icon, warning_text, space().width(Fill), install_btn,]
                .spacing(10)
                .align_y(Center)
                .padding(10),
        )
        .style(|theme| {
            let sec = container::secondary(theme);
            sec
        })
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

        let mut layout = column![self.view_header(),];

        if !self.ghostscript_found {
            layout = layout.push(self.view_ghostscript_warning());
        }

        layout
            .push(scrollable(files_list).height(Fill))
            .push(self.view_footer(files_compressed))
            .padding(25)
            .into()
    }
}

impl file_entry::FileEntry {
    fn view(&self, index: usize, running: bool) -> Element<'_, Message> {

        let document_icon = lucide_icons::iced::icon_file_text().size(25);
        let file_name     = text(self.get_file_name(50));
        let file_size     = text(file_entry::format_size(self.size)).style(text::base);

        let delete_btn = if running {
            row![button(lucide_icons::iced::icon_x()),].align_y(Center)
        } else {
            row![button(lucide_icons::iced::icon_x()).on_press(Message::RemoveFile(index)),].align_y(Center)
        };

        let finish_btn = row![
            text(file_entry::format_size(self.compressed_size)).style(text::success),
            button(lucide_icons::iced::icon_check()),
        ].align_y(Center).spacing(25);

        let content = container(
            row![
                document_icon,
                file_name,
                file_size,
                space().width(Fill),
                if self.compressed {
                    finish_btn
                } else {
                    delete_btn
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
