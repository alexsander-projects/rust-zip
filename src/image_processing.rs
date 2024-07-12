use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use image::ImageFormat;
use memmap::MmapOptions;
use tokio::fs;

static FILE_COUNT: AtomicUsize = AtomicUsize::new(1);

pub fn image_to_binary_file(image_path: &Path, output_folder: &Path) -> io::Result<PathBuf> {
    let file = File::open(image_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let binary_file_name = image_path.file_name().unwrap().to_str().unwrap().to_owned() + ".bin";
    let binary_file_path = output_folder.join(binary_file_name);

    std::fs::write(&binary_file_path, &mmap[..])?;

    let count = FILE_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("{}: Image file: {:?} converted to Binary file: {:?}", count, image_path.file_name().unwrap(), binary_file_path.file_name().unwrap());

    Ok(binary_file_path)
}

pub(crate) fn determine_image_format(binary_path: &Path) -> io::Result<ImageFormat> {
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

pub async fn convert_binary_to_image(binary_path: &Path, format: &ImageFormat, decompression_folder: &Path) -> io::Result<()> {
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
    fs::create_dir_all(&binary_files_folder).await?;

    // Move the binary file to the new folder
    let new_binary_path = binary_files_folder.join(binary_path.file_name().unwrap());
    fs::rename(binary_path, &new_binary_path).await?;

    println!("Moved binary file to: {:?}", new_binary_path);

    Ok(())
}