use tokio::fs::{self, File};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::path::{Path, PathBuf};
use serde_json::Value;
// Implement the function text_to_binary_file that reads a text file and writes its contents to a binary file.

pub async fn text_to_binary_file(text_path: &Path, output_folder: &Path) -> io::Result<PathBuf> {
    let mut text_file = File::open(text_path).await?;

    let mut contents = Vec::new();
    text_file.read_to_end(&mut contents).await?;

    let extension = text_path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or_default();

    let binary_file_name = if extension == "json" {
        text_path.file_stem().unwrap().to_str().unwrap().to_owned() + ".bin"
    } else {
        text_path.file_name().unwrap().to_str().unwrap().to_owned() + ".bin"
    };
    let binary_file_path = output_folder.join(binary_file_name);

    fs::write(&binary_file_path, &contents).await?;
    println!("File: {:?} converted to Binary file: {:?}", text_path.file_name().unwrap(), binary_file_path.file_name().unwrap());
    Ok(binary_file_path)
}

// Implement the function binary_to_text_file that reads a binary file and writes its contents to a text file.
pub async fn binary_to_text_file(binary_path: &Path, output_folder: &Path) -> io::Result<PathBuf> {
    let mut binary_file = File::open(binary_path).await?;
    let mut contents = Vec::new();
    binary_file.read_to_end(&mut contents).await?;

    let original_extension = if binary_path.file_stem().unwrap().to_str().unwrap().ends_with(".json") {
        // Attempt to parse the contents as JSON to validate structure
        let contents_str = String::from_utf8(contents.clone()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        serde_json::from_str::<Value>(&contents_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        "json"
    } else {
        "txt"
    };

    let text_file_name = format!("{}.{}", binary_path.file_stem().unwrap().to_str().unwrap(), original_extension);
    let text_file_name = text_file_name.trim_end_matches(".bin");
    let text_file_path = output_folder.join(text_file_name);

    fs::write(&text_file_path, &contents).await?;
    println!("Binary file: {:?} converted to file: {:?}", binary_path.file_name().unwrap(), text_file_path.file_name().unwrap());
    Ok(text_file_path)
}