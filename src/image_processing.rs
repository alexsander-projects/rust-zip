use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use image::ImageFormat;
use memmap::MmapOptions;

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

pub async fn convert_binary_to_image(binary_path: &Path, decompression_folder: &Path) -> io::Result<()> {
    let file = File::open(binary_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let img = image::load_from_memory(&mmap[..]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    // Determine the correct image format and extension
    let format = determine_image_format(binary_path)?;
    let extension = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        // Add more formats as needed
        _ => "png",
    };

    // Extract the file stem, removing the .bin extension if present
    let mut output_file_name = binary_path.file_stem().unwrap().to_str().unwrap().to_owned();
    if output_file_name.ends_with(extension) {
        // If the stem already ends with the correct extension, do not append again
        output_file_name.truncate(output_file_name.len() - extension.len());
    }
    let output_file_name = format!("{}.{}", output_file_name, extension);

    let output_path = decompression_folder.join(&output_file_name);

    img.save(&output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    println!("Converted binary file to image: {:?}", output_path);

    Ok(())
}