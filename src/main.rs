mod compression;
mod decompression;
mod image_processing;
mod utils;
mod text_to_binary;
mod compression_wo_conversion;
mod decompression_wo_conversion;

use std::fs::File;
use std::io::{self};
use std::path::{Path};
use std::sync::Mutex;
use zip::{ZipWriter};
use clap::Parser;

use crate::compression::add_files_to_zip;
use crate::compression::FileType;
use crate::decompression::decompress_and_convert_to_files;
use crate::compression_wo_conversion::add_files_directly_to_zip;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    Compression {
        input_folder: String,
        output_zip: String,
        #[clap(long, short, help = "Compression algorithm to use. Available options: Zstd, Bzip2, Deflated")]
        compression_algorithm: String,
        #[clap(long, short, help = "Compression level to use.\nZstd: -7 (fastest) to 22 (best compression), default 3.\nBzip2: 0 (fastest) to 9 (best compression), default 6.\nDeflated: 0 (no compression) to 9 (best compression), default 6.")]
        compression_level: i64,
        #[clap(long)]
        convert_to_binary: bool,
    },
    Decompression {
        zip_path: String,
        output_folder: String,
        #[clap(long)]
        decompress_without_conversion: bool,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Compression {
            input_folder,
            output_zip,
            compression_algorithm,
            compression_level,
            convert_to_binary,
        } => {
            let folder_path = Path::new(&input_folder);
            let output_zip_path = &output_zip;

            if !folder_path.exists() || !folder_path.is_dir() {
                println!("Error: Folder does not exist or is not a directory.");
                return Ok(());
            }

            let file = File::create(output_zip_path)?;
            let zip = ZipWriter::new(file);
            let zip_mutex = Mutex::new(zip);
            let file_type = FileType::Other; // Adjust based on your needs

            println!("Creating zip file at {}", output_zip_path);
            println!("Using compression algorithm: {}, level: {}", compression_algorithm, compression_level);

            if convert_to_binary {
                println!("Converting files to binary and adding to zip...");
                add_files_to_zip(&zip_mutex, folder_path, &compression_algorithm, compression_level, file_type)?;
            } else {
                println!("Adding files directly to zip...");
                add_files_directly_to_zip(&zip_mutex, folder_path, &compression_algorithm, compression_level)?;
            }

            let zip = zip_mutex.into_inner().unwrap();
            zip.finish()?;

            println!("Compression completed successfully.");
        },
        Commands::Decompression {
            zip_path,
            output_folder,
            decompress_without_conversion,
        } => {
            let zip_path = Path::new(&zip_path);
            let output_folder = Path::new(&output_folder);

            if decompress_without_conversion {
                println!("Decompressing without conversion...");
                decompression_wo_conversion::decompress_files(zip_path, output_folder).await?;
                println!("Decompressed file: {:?}", zip_path.file_name().unwrap());
            } else {
                println!("Decompressing and converting files...");
                decompress_and_convert_to_files(zip_path, output_folder).await?;
                println!("Decompressed and converted file: {:?}", zip_path.file_name().unwrap());
            }
        },
    }
    Ok(())
}
