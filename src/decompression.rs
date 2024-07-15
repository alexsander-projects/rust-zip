use std::fs::File;
use std::io::{self, Read};
use std::path::{Path};
use std::time::Instant;
use tokio::fs as async_fs;
use tokio::task;
use futures::future;
use zip::ZipArchive;

use crate::image_processing::{convert_binary_to_image, determine_image_format};
use crate::text_to_binary::text_to_binary_file;
use crate::text_to_binary::binary_to_text_file;

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
                    if let Ok(format) = determine_image_format(&outpath) {
                        convert_binary_to_image(&outpath, &format, &output_folder).await.unwrap();
                    }
                }
                "txt" => {
                    binary_to_text_file(&outpath, &output_folder).await.unwrap();
                }
                // "mp4" | "avi" | "mov" => {
                //     convert_binary_to_video(&outpath, &output_folder).await.unwrap();
                // }
                // "mp3" | "wav" => {
                //     convert_binary_to_audio(&outpath, &output_folder).await.unwrap();
                // }
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
    Ok(())
}
