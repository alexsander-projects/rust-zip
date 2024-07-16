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


pub fn get_file_type(extension: &str) -> FileType {
    match extension {
        "png" | "jpg" | "gif" | "jpeg" => FileType::Image,
        "mp4" | "avi" | "mov"=> FileType::Video,
        "mp3" | "wav" => FileType::Audio,
        "txt" => FileType::Text,
        _ => FileType::Other,
    }
}

pub fn file_type_matches(path: &Path, file_type: &FileType) -> bool {
    match file_type {
        FileType::Image => path.extension().map_or(false, |ext| ext == "png" || ext == "jpg"),
        FileType::Video => path.extension().map_or(false, |ext| ext == "mp4" || ext == "avi"),
        FileType::Audio => path.extension().map_or(false, |ext| ext == "mp3" || ext == "wav"),
        FileType::Text => path.extension().map_or(false, |ext| ext == "txt"),
        FileType::Other => true, // Or implement specific logic for other file types
    }
}

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
