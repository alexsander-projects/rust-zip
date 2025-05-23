use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;
use tokio::fs as async_fs;
use tokio::task;
use futures::future;
use zip::ZipArchive;

/// Decompresses files from a ZIP archive directly to the output folder without any conversion.
///
/// This function reads a ZIP archive and extracts each file to the specified `output_folder`.
/// It processes files asynchronously for potentially improved performance.
///
/// # Arguments
///
/// * `zip_path` - The path to the ZIP file to be decompressed.
/// * `output_folder` - The directory where the decompressed files will be saved.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or failure of the operation.
pub async fn decompress_files(zip_path: &Path, output_folder: &Path) -> io::Result<()> {
    let start = Instant::now();
    println!("Starting decompression process...");
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

        tasks.push(task::spawn(async move {
            async_fs::write(&outpath, &buffer).await.unwrap();
            println!("Extracted file: {:?}", outpath.file_name().unwrap());
        }));
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
    future::join_all(tasks).await;
    println!("Decompression process completed.");
    Ok(())
}

