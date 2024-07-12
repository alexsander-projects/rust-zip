use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use rayon::prelude::*;
use zip::{write::FileOptions, ZipWriter};
use crate::image_processing::image_to_binary_file;
use crate::utils::get_compression_method;

pub fn add_binary_files_to_zip(
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