use std::path::PathBuf;
use std::fs;
use rfd::AsyncFileDialog;

use crate::file_entry::FileEntry;

pub async fn select_output_folder() -> PathBuf {
    let output_path: Option<rfd::FileHandle> = AsyncFileDialog::new()
        .set_title("Select Output Folder")
        .pick_folder()
        .await;

    match output_path {
        Some(folder) => folder.path().to_path_buf(),
        None => PathBuf::new(), // TODO better handling
    }
}

pub async fn open_file_selection() -> Vec<FileEntry> {
    let selected_files: Option<Vec<rfd::FileHandle>> = AsyncFileDialog::new()
        .add_filter("Pdf", &["pdf"])
        .add_filter("All", &["*"])
        .set_title("Select documents to compress")
        .pick_files()
        .await;

    // Formatting into struct...
    selected_files.iter()
        .flatten()
        .filter_map(|files_handle| {
            if !files_handle.path().is_file() {
                None
            } else {
                let path = files_handle
                                    .path()
                                    .to_path_buf();
                let size = fs::metadata(&path)
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                Some(FileEntry { path, size, compressed_size:0, compressed:false })
            }

    }).collect()
}
