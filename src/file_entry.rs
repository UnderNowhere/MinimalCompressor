use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path:PathBuf,
    pub size:u64,           // TODO
    pub compressed_size:u64,
    pub compressed:bool,
}

impl FileEntry {
    pub fn get_file_name(&self, max_size:usize) -> String {
        let mut file_name = match self.path.file_name() {
            Some(name) => name.to_str().unwrap_or("unknown").to_string(),
            None => "unknown".to_string()
        };

        if file_name.len() > max_size {
            file_name.truncate(max_size - 3);
            file_name.push_str("...");
            file_name
        } else {
            file_name
        }
    }
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
