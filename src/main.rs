use std::fs::{self, File};
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::ZipArchive;
use std::time::Instant;
use rayon::prelude::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use image::{ImageFormat};
use memmap::MmapOptions;

static FILE_COUNT: AtomicUsize = AtomicUsize::new(1);

fn image_to_binary_file(image_path: &Path, output_folder: &Path) -> io::Result<PathBuf> {
    let file = File::open(image_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let binary_file_name = image_path.file_name().unwrap().to_str().unwrap().to_owned() + ".bin";
    let binary_file_path = output_folder.join(binary_file_name);

    fs::write(&binary_file_path, &mmap[..])?;

    let count = FILE_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("{}: Image file: {:?} converted to Binary file: {:?}", count, image_path.file_name().unwrap(), binary_file_path.file_name().unwrap());

    Ok(binary_file_path)
}

fn decompress_and_convert_to_images(zip_path: &Path, output_folder: &Path) -> io::Result<()> {
    fs::create_dir_all(output_folder)?;

    let file = File::open(zip_path)?;
    let archive = Mutex::new(ZipArchive::new(file)?);

    let archive_len = archive.lock().unwrap().len();
    println!("Archive contains {} entries", archive_len);

    if archive_len == 0 {
        println!("No entries to decompress.");
        return Ok(());
    }

    (0..archive_len).into_par_iter().for_each(|i| {
        let mut archive = archive.lock().unwrap();
        let file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                println!("Failed to read file index {} from archive: {:?}", i, e);
                return;
            }
        };

        let outpath = match file.enclosed_name() {
            Some(path) => output_folder.join(path),
            None => {
                println!("Skipping file at index {}: invalid file name", i);
                return;
            }
        };

        println!("Decompressing: {:?}", file.name());

        if file.name().ends_with('/') {
            if let Err(e) = fs::create_dir_all(&outpath) {
                println!("Failed to create directory {:?}: {:?}", outpath, e);
            }
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(&p) {
                        println!("Failed to create directory for file {:?}: {:?}", p, e);
                        return;
                    }
                }
            }
            let mut outfile = match File::create(&outpath) {
                Ok(file) => file,
                Err(e) => {
                    println!("Failed to create output file {:?}: {:?}", outpath, e);
                    return;
                }
            };
            let file_size = file.size();
            if let Err(e) = io::copy(&mut file.take(file_size), &mut outfile) {
                println!("Failed to decompress file {:?}: {:?}", outpath, e);
            } else {
                println!("Decompressed: {:?}", outpath);
                // Attempt to convert the binary file to an image
                match determine_image_format(&outpath) {
                    Ok(format) => {
                        if let Err(e) = convert_binary_to_image(&outpath, &format) {
                            println!("Failed to convert to image: {:?}, error: {:?}", outpath, e);
                        } else {
                            println!("Converted to image: {:?}", outpath);
                        }
                    },
                    Err(e) => println!("Failed to determine image format for {:?}: {:?}", outpath, e),
                }
            }
        }
    });

    Ok(())
}


fn determine_image_format(binary_path: &Path) -> io::Result<ImageFormat> {
    let mut extension = binary_path.extension().and_then(std::ffi::OsStr::to_str);

    // Check if the extension is .bin, and if so, strip it and re-evaluate the extension
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

fn convert_binary_to_image(binary_path: &Path, format: &ImageFormat) -> io::Result<()> {
    let file = File::open(binary_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let img = image::load_from_memory(&mmap[..])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let output_path = binary_path.with_extension(match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => "png",
    });

    img.save(output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

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
    fs::create_dir_all(&output_folder)?;

    let entries: Vec<_> = fs::read_dir(folder_path)?.filter_map(|e| e.ok()).collect();

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
                    if io::copy(&mut file, &mut *zip_guard).is_err() {
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

fn main() -> io::Result<()> {
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


    decompress_and_convert_to_images(zip_path, output_folder)?;
    Ok(())
}