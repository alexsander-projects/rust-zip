use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use futures::future;
use image::ImageFormat;
use memmap::MmapOptions;
use rayon::prelude::*;
use tokio::fs as async_fs;
use tokio::task;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

static FILE_COUNT: AtomicUsize = AtomicUsize::new(1);

fn image_to_binary_file(image_path: &Path, output_folder: &Path) -> io::Result<PathBuf> {
    let file = File::open(image_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let binary_file_name = image_path.file_name().unwrap().to_str().unwrap().to_owned() + ".bin";
    let binary_file_path = output_folder.join(binary_file_name);

    std::fs::write(&binary_file_path, &mmap[..])?;

    let count = FILE_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("{}: Image file: {:?} converted to Binary file: {:?}", count, image_path.file_name().unwrap(), binary_file_path.file_name().unwrap());

    Ok(binary_file_path)
}

async fn decompress_and_convert_to_images(zip_path: &Path, output_folder: &Path) -> io::Result<()> {
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

            if let Ok(format) = determine_image_format(&outpath) {
                convert_binary_to_image(&outpath, &format, &output_folder).unwrap();
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

fn determine_image_format(binary_path: &Path) -> io::Result<ImageFormat> {
    let mut extension = binary_path.extension().and_then(std::ffi::OsStr::to_str);

    if extension == Some("bin") {
        if let Some(stem) = binary_path.file_stem().and_then(|s| s.to_str()) {
            if let Some(pos) = stem.rfind('.') {
                extension = Some(&stem[(pos + 1)..]);
            }
        }
    }

    match extension {
        Some("png") | Some("Png") => Ok(ImageFormat::Png),
        Some("jpg") | Some("jpeg") | Some("Jpg") | Some("Jpeg") => Ok(ImageFormat::Jpeg),
        Some("gif") | Some("Gif") => Ok(ImageFormat::Gif),
        Some("webp") | Some("Webp") => Ok(ImageFormat::WebP),
        Some("tiff") | Some("tif") | Some("Tiff") | Some("Tif") => Ok(ImageFormat::Tiff),
        Some("bmp") | Some("Bmp") => Ok(ImageFormat::Bmp),
        Some("ico") | Some("Ico") => Ok(ImageFormat::Ico),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsupported or unknown image format",
        )),
    }
}

fn convert_binary_to_image(binary_path: &Path, format: &ImageFormat, decompression_folder: &Path) -> io::Result<()> {
    let file = File::open(binary_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let img = image::load_from_memory(&mmap[..])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let output_path = binary_path.with_extension(match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => "png",
    });

    img.save(&output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Define the new folder path for binary files
    let binary_files_folder = decompression_folder.join("binary_files");
    fs::create_dir_all(&binary_files_folder)?;

    // Move the binary file to the new folder
    let new_binary_path = binary_files_folder.join(binary_path.file_name().unwrap());
    fs::rename(binary_path, &new_binary_path)?;

    println!("Moved binary file to: {:?}", new_binary_path);

    Ok(())
}

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

fn add_binary_files_to_zip(
    zip: &Mutex<ZipWriter<File>>,
    folder_path: &Path,
    compression_algorithm: &str,
    compression_level: i64,
) -> io::Result<()> {
    let output_folder = folder_path.join("binary_output");
    std::fs::create_dir_all(&output_folder)?;

    let entries: Vec<_> = std::fs::read_dir(folder_path)?.filter_map(|e| e.ok()).collect();

    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() && (path.extension().map_or(false, |ext| ext == "png" || ext == "jpg")) {
            let binary_file_path = match image_to_binary_file(&path, &output_folder) {
                Ok(path) => path,
                Err(e) => {
                    println!("Error converting image to binary: {:?}", e);
                    return;
                }
            };

            let relative_path = binary_file_path.strip_prefix(&output_folder).unwrap_or(&binary_file_path);
            let (compression_method, valid_level) = match get_compression_method(compression_algorithm, compression_level) {
                Ok((method, level)) => (method, level),
                Err(e) => {
                    println!("Error getting compression method: {:?}", e);
                    return;
                }
            };

            let options: FileOptions<()> = FileOptions::default()
                .compression_method(compression_method).compression_level(Option::from(valid_level));

            let file_name = match relative_path.to_str() {
                Some(name) => name,
                None => {
                    println!("Error getting file name: {:?}", relative_path);
                    return;
                }
            };

            let mut zip_guard = zip.lock().unwrap();
            match zip_guard.start_file(file_name, options) {
                Ok(_) => {
                    let mut file = match File::open(&binary_file_path) {
                        Ok(file) => file,
                        Err(e) => {
                            println!("Error opening binary file: {:?}", e);
                            return;
                        }
                    };
                    if std::io::copy(&mut file, &mut *zip_guard).is_err() {
                        println!("Error adding binary file to zip: {}", file_name);
                    }
                },
                Err(e) => println!("Error starting file in zip: {}, {:?}", file_name, e),
            }
        } else {
            println!("Skipping non-image file or directory: {:?}", path);
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        println!("Usage: <input_folder> <output_zip> <compression_algorithm> <compression_level>");
        return Ok(());
    }

    let zip_path = Path::new("C:/Users/Alexs/RustroverProjects/compressimagesvideosrust/output.zip");
    let output_folder = Path::new("C:/Users/Alexs/Desktop/testsetsetsetse");
    let folder_path = Path::new(&args[1]);
    let output_zip_path = &args[2];
    let compression_algorithm = &args[3];
    let compression_level = args[4].parse::<i64>().unwrap_or(3); // Default level to 3 if parsing fails

    if !folder_path.exists() || !folder_path.is_dir() {
        println!("Error: Folder does not exist or is not a directory.");
        return Ok(());
    }

    let file = File::create(output_zip_path)?;
    let zip = ZipWriter::new(file);
    let zip_mutex = Mutex::new(zip);

    let start = Instant::now();

    println!("Creating zip file at {}", output_zip_path);
    println!("Using compression algorithm: {}, level: {}", compression_algorithm, compression_level);

    add_binary_files_to_zip(&zip_mutex, folder_path, compression_algorithm, compression_level)?;

    let zip = zip_mutex.into_inner().unwrap();
    zip.finish()?;

    let duration = start.elapsed();
    println!("Zip file created successfully at {}", output_zip_path);
    println!("Time taken: {} ms", duration.as_millis());

    decompress_and_convert_to_images(zip_path, output_folder).await?;
    Ok(())
}
