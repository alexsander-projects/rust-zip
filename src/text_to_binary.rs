use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use memmap::MmapOptions;
use std::sync::atomic::{AtomicUsize, Ordering};

static FILE_COUNT: AtomicUsize = AtomicUsize::new(1);

/// Converts a text file to a binary file format.
///
/// This function reads a text file, memory-maps its contents, and writes the raw bytes
/// to a new file with a `.bin` extension in the specified `output_folder`.
/// It also prints a message indicating the conversion, including a unique count for each processed file.
///
/// # Arguments
///
/// * `text_path` - The path to the input text file.
/// * `output_folder` - The directory where the binary file will be saved.
///
/// # Returns
///
/// An `io::Result<PathBuf>` containing the path to the created binary file, or an error.
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

/// Determines the original format (e.g., "json", "txt") of a binary file.
///
/// This function inspects the file stem of the binary path (e.g., `document.txt.bin` -> `document.txt`)
/// to infer the original text format.
///
/// # Arguments
///
/// * `binary_path` - The path to the binary file (e.g., `data.json.bin`).
///
/// # Returns
///
/// An `io::Result<String>` representing the determined text format (e.g., "json", "txt"),
/// or an error if unsupported/unknown.
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

/// Converts a binary file back to a text file.
///
/// This function reads a binary file (assumed to be a previously converted text file),
/// decodes its content as UTF-8 (lossy), determines its original format (e.g., ".txt", ".json"),
/// and saves it as a text file in the `decompression_folder` with the appropriate extension.
///
/// # Arguments
///
/// * `binary_path` - The path to the binary file.
/// * `decompression_folder` - The directory where the converted text file will be saved.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or failure of the conversion.
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

