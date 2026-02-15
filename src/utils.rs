use std::path::PathBuf;
use tokio::process::Command;
use rfd::AsyncFileDialog;
use std::fs;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path:PathBuf,
    pub size:u64,           // TODO
    pub compressed_size:u64,
    pub compressed:bool,
}

// pub fn dectect_gs() -> bool {
//     // TODO
//     false
// }

pub fn format_output_file(path: &PathBuf, output_path:&PathBuf, quality_parm: &String) -> PathBuf {
    output_path.join(
        format!(
            "{}_{}.pdf",
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown"),
            quality_parm
        )
    )
}

// ─── Initial Fake compressor worker ───
// note: Very dangerous returns TODO chose a Map struct (file with an id)
// to avoid index deletion during the working async task... 
pub async fn compress_pdf(path: PathBuf, index: usize, quality_parm: String, output_path: PathBuf) -> usize {
    // TODO proper conversion and more robust. should be an exteranl utils function... (right now ugly)
    // TODO think about the given type... does PathBuf is usefull ? 
    let output_file = format_output_file(&path, &output_path, &quality_parm);

    let mut cmd = Command::new("gs");
    cmd.arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.4")
        .arg(format!("-dPDFSETTINGS=/{}", quality_parm))
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH")
        .arg(format!("-sOutputFile={}", output_file.display()))
        .arg(&path);

    println!("Wrinting file with compression quality ({}) at: {}", quality_parm, output_file.display());

    let output = cmd.output().await;
    
    println!("Compression status: {:?}", output.map(|f| f.status));

    index   // to keep track of which one has been converted
}

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
        .map(|files_handle| {
            let path = files_handle
                                .path()
                                .to_path_buf(); 
            let size = fs::metadata(&path)
                                .map(|m| m.len())
                                .unwrap_or(0);
            FileEntry { path, size, compressed_size:0, compressed:false }
    }).collect()
}

pub fn format_size(size:u64) -> String {
    if size > 1000*1000 {
        let _size = size as f64 / (1000.0 * 1000.0);
        format!("{:.2} MB", _size).to_string()
    } else {
        let _size = size as f64 / 1000.0;
        format!("{:.2} KB", _size).to_string()

    }
}