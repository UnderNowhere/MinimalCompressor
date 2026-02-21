use std::path::PathBuf;
use tokio::process::Command;

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
