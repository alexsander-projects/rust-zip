use std::fs::{self, File};
use std::io;
use std::path::{Path};
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;
use std::env;
use std::time::Instant;

fn get_compression_method(algorithm: &str, level: i64) -> io::Result<(CompressionMethod, Option<i64>)> {
    match algorithm {
        "Zstd" => {
            let valid_level = if level >= -7 && level <= 22 { Some(level) } else { Some(3) };
            Ok((CompressionMethod::Zstd, valid_level))
        },
        "Bzip2" => {
            let valid_level = if level >= 0 && level <= 9 { Some(level) } else { Some(6) };
            Ok((CompressionMethod::Bzip2, valid_level))
        },
        "Deflated" => {
            let valid_level = if level >= 0 && level <= 9 { Some(level) }
            else { Some(6) };
            Ok((CompressionMethod::Deflated, valid_level))
        },
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported compression algorithm, supported algorithms are: Zstd, Bzip2, Deflated")),
    }
}

fn add_folder_contents_to_zip(
    zip: &mut ZipWriter<File>,
    folder_path: &Path,
    base_folder_name: &str,
    compression_algorithm: &str,
    compression_level: i64,
) -> io::Result<()> {
    for entry in fs::read_dir(folder_path)? {
        let (compression_method, valid_level) = get_compression_method(compression_algorithm, compression_level)?;

        let entry = entry?;
        let path = entry.path();
        let relative_path = match path.strip_prefix(base_folder_name) {
            Ok(rp) => rp,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to strip prefix: {}", e),
                ));
            }
        };
        if path.is_dir() {
            add_folder_contents_to_zip(zip, &path, base_folder_name, compression_algorithm, compression_level)?;
        } else {
            let options: FileOptions<()> = FileOptions::default()
                .compression_method(compression_method).compression_level(Option::from(valid_level));
            let file_name = relative_path.to_str().unwrap();
            zip.start_file(file_name, options)?;
            let mut file = File::open(&path)?;
            io::copy(&mut file, zip)?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        println!("Usage: <input_folder> <output_zip> <compression_algorithm> <compression_level>");
        return Ok(());
    }
    let folder_path = Path::new(&args[1]);
    let output_zip_path = &args[2];
    let compression_algorithm = &args[3];
    let compression_level = args[4].parse::<i64>().unwrap();

    if !folder_path.exists() || !folder_path.is_dir() {
        println!("Error: Folder does not exist or is not a directory.");
        return Ok(());
    }

    let file = File::create(output_zip_path)?;
    let mut zip = ZipWriter::new(file);

    let start = Instant::now();

    println!("Creating zip file at {}", output_zip_path);
    println!("Using compression algorithm: {}, level: {}", compression_algorithm, compression_level);

    add_folder_contents_to_zip(&mut zip, folder_path, folder_path.to_str().unwrap(), compression_algorithm, compression_level)?;

    zip.finish()?;
    println!("Zip file created successfully at {}", output_zip_path);
    println!("Time taken: {} ms", start.elapsed().as_millis());

    Ok(())
}
