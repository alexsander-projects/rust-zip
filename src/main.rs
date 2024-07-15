mod compression;
mod decompression;
mod image_processing;
mod utils;
mod text_to_binary;

use std::fs::File;
use std::io::{self};
use std::path::{Path};
use std::sync::Mutex;
use std::time::Instant;
use zip::{ZipWriter};

use crate::compression::add_files_to_zip;
use crate::compression::FileType;
use crate::decompression::decompress_and_convert_to_files;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("compression") => {
            if args.len() != 6 {
                println!("Usage for compression: cargo run -- compression <input_folder> <output_zip> <compression_algorithm> <compression_level>");
                return Ok(());
            }
            let folder_path = Path::new(&args[2]);
            let output_zip_path = &args[3];
            let compression_algorithm = &args[4];
            let compression_level = args[5].parse::<i64>().unwrap_or(3); // Default level to 3 if parsing fails

            if !folder_path.exists() || !folder_path.is_dir() {
                println!("Error: Folder does not exist or is not a directory.");
                return Ok(());
            }

            let file = File::create(output_zip_path)?;
            let zip = ZipWriter::new(file);
            let zip_mutex = Mutex::new(zip);
            let file_type = FileType::Other;

            let start = Instant::now();

            println!("Creating zip file at {}", output_zip_path);
            println!("Using compression algorithm: {}, level: {}", compression_algorithm, compression_level);

            add_files_to_zip(&zip_mutex, folder_path, compression_algorithm, compression_level, file_type)?;

            let zip = zip_mutex.into_inner().unwrap();
            zip.finish()?;

            println!("File {} added to zip", folder_path.display());

            let duration = start.elapsed();
            println!("Zip file created successfully at {}", output_zip_path);
            println!("Time taken: {} ms", duration.as_millis());
        },
        Some("decompression") => {
            if args.len() != 4 {
                println!("Usage for decompression: cargo run -- decompression <zip_path> <output_folder>");
                return Ok(());
            }
            let zip_path = Path::new(&args[2]);
            let output_folder = Path::new(&args[3]);

            decompress_and_convert_to_files(zip_path, output_folder).await?;

            println!("Decompressed file: {:?}", zip_path.file_name().unwrap());
        },
        _ => println!("Invalid mode. Please specify 'compression' or 'decompression'."),
    }
    Ok(())
}