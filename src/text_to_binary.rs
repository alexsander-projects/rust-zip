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

pub async fn determine_text_format(binary_path: &Path) -> io::Result<String> {
    let mut extension = binary_path.extension().and_then(std::ffi::OsStr::to_str);

    if extension == Some("bin") {
        if let Some(stem) = binary_path.file_stem().and_then(|s| s.to_str()) {
            if let Some(pos) = stem.rfind('.') {
                extension = Some(&stem[(pos + 1)..]);
            }
        }
    }

    match extension {
        Some("json") | Some("Json") => Ok("json".to_string()),
        Some("txt") | Some("Txt") => Ok("txt".to_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsupported or unknown text format",
        )),
    }
}

// Implement the function binary_to_text_file that reads a binary file and writes its contents to a text file.
pub async fn convert_binary_to_text(binary_path: &Path, decompression_folder: &Path) -> io::Result<()> {
    let async_file = fs::File::open(binary_path).await?;
    let std_file = async_file.into_std().await;
    let mmap = unsafe { MmapOptions::new().map(&std_file)? };
    let text_content = std::str::from_utf8(&mmap)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    // Extract the file stem, removing the .bin extension if present
    let mut output_file_name = binary_path.file_stem().unwrap().to_str().unwrap().to_owned();
    if output_file_name.ends_with(".txt") {
        output_file_name.truncate(output_file_name.len() - 4); // Remove .txt if present
    }
    output_file_name.push_str(".txt"); // Append .txt extension
    let output_path = decompression_folder.join(output_file_name);

    fs::write(&output_path, text_content).await?;

    // Define the new folder path for binary files
    let binary_files_folder = decompression_folder.join("binary_files");
    fs::create_dir_all(&binary_files_folder).await?;

    // Move the binary file to the new folder
    let new_binary_path = binary_files_folder.join(binary_path.file_name().unwrap());
    fs::rename(binary_path, &new_binary_path).await?;

    println!("Moved binary file to: {:?}", new_binary_path);
    println!("Converted binary file to text: {:?}", output_path);

    Ok(())
}