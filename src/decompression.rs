use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;
use tokio::fs as async_fs;
use tokio::fs::{self, remove_file, read_dir};
use tokio::task;
use futures::future;
use zip::ZipArchive;
use std::ffi::OsStr;

use crate::image_processing::{convert_binary_to_image, determine_image_format};
use crate::text_to_binary::convert_binary_to_text;

#[derive(Debug)]
pub enum FileType {
    Image,
    Video,
    Audio,
    Text,
    Other,
}

pub async fn decompress_and_convert_to_files(zip_path: &Path, output_folder: &Path) -> io::Result<()> {
    println!("Starting decompression and conversion process...");
    let overall_start = Instant::now();
    async_fs::create_dir_all(output_folder).await?;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    let archive_len = archive.len();
    println!("Archive contains {} entries", archive_len);

    if archive_len == 0 {
        println!("No entries to decompress.");
        return Ok(());
    }

    let mut tasks = vec![];

    for i in 0..archive_len {
        let start = Instant::now();
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => output_folder.join(path),
            None => {
                println!("Skipping file at index {}: invalid file name", i);
                continue;
            }
        };

        println!("Processing file at index {}: {:?}", i, outpath.file_name().unwrap());

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let output_folder = output_folder.to_path_buf();

        tasks.push(task::spawn(async move {
            async_fs::write(&outpath, &buffer).await.unwrap();

            let extension = outpath.extension().and_then(|ext| ext.to_str()).unwrap_or_default();

            match extension {
                "bin" => {
                    if let Ok(_format) = determine_image_format(&outpath) {
                        convert_binary_to_image(&outpath, &output_folder).await.unwrap();
                        tokio::fs::remove_file(&outpath).await.unwrap();
                        println!("Removed binary file: {:?}", outpath.file_name().unwrap());
                    } else {
                        // Handle non-image binary files appropriately
                        let content_type = determine_file_type(&outpath);
                        match content_type {
                            FileType::Text => {
                                convert_binary_to_text(&outpath, &output_folder).await.unwrap();
                                let _=tokio::fs::remove_file(&outpath).await;
                                println!("Removed binary file: {:?}", outpath.file_name().unwrap());
                            }
                            _ => println!("Unsupported binary file type: {:?}", content_type),
                        }
                    }
                }
                "txt" => {
                    convert_binary_to_text(&outpath, &output_folder).await.unwrap();
                }
                _ => {
                    println!("Unsupported file format: {}", extension);
                }
            }

            let duration = start.elapsed();
            println!("File processed in {} ms", duration.as_millis());
        }));
    }

    future::join_all(tasks).await;
    let overall_duration = overall_start.elapsed();
    println!("Decompression and conversion process completed in {} ms", overall_duration.as_millis());
    delete_remaining_bin_files(output_folder).await?;
    println!("Removed remaining binary files");
    Ok(())
}

async fn delete_remaining_bin_files(output_folder:&Path) -> io::Result<()> {
    let binary_files_folder = output_folder.join("binary_files");
    if binary_files_folder.exists(){
        let mut entries = read_dir(binary_files_folder.clone()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                remove_file(path).await?;
            }
        }
        fs::remove_dir(binary_files_folder).await?;
    }
    Ok(())
}

fn determine_file_type(path: &Path) -> FileType {
    let extension = path.extension().and_then(OsStr::to_str);
    let mut file_type = FileType::Other;

    if extension == Some("bin") {
        if let Some(stem) = path.file_stem().and_then(OsStr::to_str) {
            if stem.ends_with(".txt") {
                file_type = FileType::Text;
            } else if stem.ends_with(".mp4") || stem.ends_with(".avi") || stem.ends_with(".mov") {
                file_type = FileType::Video;
            } else if stem.ends_with(".mp3") || stem.ends_with(".wav") {
                file_type = FileType::Audio;
            } else if stem.ends_with(".png") || stem.ends_with(".jpg") || stem.ends_with(".jpeg") {
                file_type = FileType::Image;
            }
        }
    } else {
        match extension {
            Some("txt") | Some("json") => file_type = FileType::Text,
            Some("mp4") | Some("avi") | Some("mov") => file_type = FileType::Video,
            Some("mp3") | Some("wav") => file_type = FileType::Audio,
            Some("png") | Some("jpg") | Some("jpeg") => file_type = FileType::Image,
            _ => file_type = FileType::Other,
        }
    }

    file_type
}