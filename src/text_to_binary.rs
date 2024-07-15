use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use memmap::MmapOptions;
use std::sync::atomic::{AtomicUsize, Ordering};
// Implement the function text_to_binary_file that reads a text file and writes its contents to a binary file.

static FILE_COUNT: AtomicUsize = AtomicUsize::new(1);

pub fn text_to_binary_file(text_path: &Path, output_folder: &Path) -> io::Result<PathBuf> {
    let file = File::open(text_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let binary_file_name = text_path.file_name().unwrap().to_str().unwrap().to_owned() + ".bin";
    let binary_file_path = output_folder.join(binary_file_name);

    std::fs::write(&binary_file_path, &mmap[..])?;

    let count = FILE_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("{}: Text file: {:?} converted to Binary file: {:?}", count, text_path.file_name().unwrap(), binary_file_path.file_name().unwrap());

    Ok(binary_file_path)
}

pub async fn determine_text_format(binary_path: &Path) -> io::Result<String> {
    let mut extension = binary_path.extension().and_then(std::ffi::OsStr::to_str);

    if extension == Some("bin") {
        if let Some(stem) = binary_path.file_stem().and_then(|s| s.to_str()) {
            if let Some(pos) = stem.rfind('.') {
                extension = Some(&stem[(pos + 1)..]);
            }
        }
    }

    match extension {
        Some("json") | Some("Json") => Ok("json".to_string()),
        Some("txt") | Some("Txt") => Ok("txt".to_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsupported or unknown text format",
        )),
    }
}

// Implement the function binary_to_text_file that reads a binary file and writes its contents to a text file.
pub async fn convert_binary_to_text(binary_path: &Path, decompression_folder: &Path) -> io::Result<()> {
    let file = File::open(binary_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let txt = String::from_utf8_lossy(&mmap[..]);

    // Determine the correct image format and extension
    let format = determine_text_format(binary_path).await?;
    let extension = match format.as_str() {
        //expect String
        "json" => "json".to_string(),
        "txt" => "txt".to_string(),
        _=> "txt".to_string(),
    };

    // Extract the file stem, removing the .bin extension if present
    let mut output_file_name = binary_path.file_stem().unwrap().to_str().unwrap().to_owned();
    if output_file_name.ends_with(&extension) {
        // If the stem already ends with the correct extension, do not append again
        output_file_name.truncate(output_file_name.len() - extension.len());
    }
    let output_file_name = format!("{}.{}", output_file_name, extension);

    let output_path = decompression_folder.join(&output_file_name);

    //change this to another function supporting txt
    //img.save(&output_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    std::fs::write(&output_path, &txt[..])?;

    println!("Converted binary file to txt: {:?}", output_path);

    Ok(())
}