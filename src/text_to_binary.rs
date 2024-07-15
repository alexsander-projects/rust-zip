use tokio::fs::{self, File};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::path::{Path, PathBuf};
use serde_json::Value;
use memmap::MmapOptions;
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
pub async fn convert_binary_to_text(binary_path: &Path, decompression_folder: &Path) -> io::Result<()> {
    // Open the file asynchronously
    let async_file = fs::File::open(binary_path).await?;

    // Convert the async file to a standard file
    let std_file = async_file.into_std().await;

    // Memory map the file
    let mmap = unsafe { MmapOptions::new().map(&std_file)? };

    // Convert the binary content to text
    let text_content = std::str::from_utf8(&mmap)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    // Define the output path with .txt extension
    let output_path = binary_path.with_extension("txt");

    // Write the text content to the output path asynchronously
    fs::write(&output_path, text_content).await?;

    // Define the new folder path for binary files
    let binary_files_folder = decompression_folder.join("binary_files");
    fs::create_dir_all(&binary_files_folder).await?;

    // Move the binary file to the new folder
    let new_binary_path = binary_files_folder.join(binary_path.file_name().unwrap());
    fs::rename(binary_path, &new_binary_path).await?;

    println!("Moved binary file to: {:?}", new_binary_path);

    Ok(())
}