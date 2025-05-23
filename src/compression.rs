use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::path::PathBuf;
use std::time::Instant;
use rayon::prelude::*;
use zip::{write::FileOptions, ZipWriter};

use crate::image_processing::image_to_binary_file;
use crate::text_to_binary::text_to_binary_file;
use crate::utils::get_compression_method;

pub enum FileType {
    Image,
    Video,
    Audio,
    Text,
    Other,
}

/// Adds files to a ZIP archive, optionally converting them to a target format before compression.
///
/// This function iterates over files in the `folder_path`, converts them if necessary based on `file_type`,
/// compresses them using the specified `compression_algorithm` and `compression_level`,
/// and adds them to the `zip` archive.
///
/// # Arguments
///
/// * `zip` - A thread-safe `Mutex` wrapping a `ZipWriter` for the output archive.
/// * `folder_path` - The path to the folder containing files to be compressed.
/// * `compression_algorithm` - The name of the compression algorithm to use.
/// * `compression_level` - The level of compression to apply.
/// * `file_type` - The type of files to process, influencing potential conversion steps.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or failure of the operation.
pub fn add_files_to_zip(
    zip: &Mutex<ZipWriter<File>>,
    folder_path: &Path,
    compression_algorithm: &str,
    compression_level: i64,
    file_type: FileType,
) -> io::Result<()> {
    let output_folder = folder_path.join("output");
    let start = Instant::now();
    std::fs::create_dir_all(&output_folder)?;

    let entries: Vec<_> = std::fs::read_dir(folder_path)?.filter_map(|e| e.ok()).collect();

    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() && file_type_matches(&path, &file_type) {
            match convert_to_target_format(&path, &output_folder, &file_type) {
                Ok(output_file_path) => {
                    let file_name = output_file_path.file_name().unwrap().to_str().unwrap();

                    let (compression_method, valid_level) = match get_compression_method(compression_algorithm, compression_level) {
                        Ok((method, level)) => (method, level),
                        Err(e) => {
                            println!("Error getting compression method: {:?}", e);
                            return;
                        }
                    };

                    let options: FileOptions<()> = FileOptions::default()
                        .compression_method(compression_method).compression_level(Option::from(valid_level));

                    let mut zip_guard = zip.lock().unwrap();
                    match zip_guard.start_file(file_name, options) {
                        Ok(_) => {
                            let mut file = match File::open(&output_file_path) {
                                Ok(file) => file,
                                Err(e) => {
                                    println!("Error opening file: {:?}", e);
                                    return;
                                }
                            };
                            if std::io::copy(&mut file, &mut *zip_guard).is_err() {
                                println!("Error adding file to zip: {}", file_name);
                            }
                        },
                        Err(e) => println!("Error starting file in zip: {}, {:?}", file_name, e),
                    }
                }
                Err(e) => {
                    println!("Error converting file: {:?}", e);
                }
            }
        } else {
            println!("Skipping non-target file or directory: {:?}", path);
        }
    });

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
    Ok(())
}

/// Determines the `FileType` of a file based on its extension.
///
/// # Arguments
///
/// * `extension` - A string slice representing the file extension.
///
/// # Returns
///
/// A `FileType` enum variant corresponding to the extension.
pub fn get_file_type(extension: &str) -> FileType {
    match extension {
        "png" | "jpg" | "gif" | "jpeg" => FileType::Image,
        "mp4" | "avi" | "mov"=> FileType::Video,
        "mp3" | "wav" => FileType::Audio,
        "txt" => FileType::Text,
        _ => FileType::Other,
    }
}

/// Checks if a file at a given path matches a specified `FileType`.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `file_type` - The `FileType` to check against.
///
/// # Returns
///
/// `true` if the file matches the `file_type`, `false` otherwise.
pub fn file_type_matches(path: &Path, file_type: &FileType) -> bool {
    match file_type {
        FileType::Image => path.extension().map_or(false, |ext| ext == "png" || ext == "jpg"),
        FileType::Video => path.extension().map_or(false, |ext| ext == "mp4" || ext == "avi"),
        FileType::Audio => path.extension().map_or(false, |ext| ext == "mp3" || ext == "wav"),
        FileType::Text => path.extension().map_or(false, |ext| ext == "txt"),
        FileType::Other => true, // Or implement specific logic for other file types
    }
}

/// Converts a file to a target format based on its `FileType`.
///
/// For images and text files, this function may convert them to a binary representation.
/// Other file types are typically copied as is.
///
/// # Arguments
///
/// * `path` - The path to the input file.
/// * `output_folder` - The directory where the converted file will be saved.
/// * `_file_type` - The `FileType` of the input file (currently used to determine the target format indirectly via `get_file_type`).
///
/// # Returns
///
/// An `io::Result<PathBuf>` containing the path to the converted file, or an error.
pub fn convert_to_target_format(path: &Path, output_folder: &Path, _file_type: &FileType) -> io::Result<PathBuf> {
    let extension = path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or_default();
    let target_file_type = get_file_type(extension);

    match target_file_type {
        FileType::Image => image_to_binary_file(path, output_folder),
        FileType::Text => text_to_binary_file(path, output_folder),
        FileType::Video | FileType::Audio | FileType::Other => {
            let output_path = output_folder.join(path.file_name().unwrap());
            std::fs::copy(path, &output_path)?;
            Ok(output_path)
        }
    }
}
