use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::path::PathBuf;
use rayon::prelude::*;
use zip::{write::FileOptions, ZipWriter};

use crate::image_processing::image_to_binary_file;
use crate::text_to_binary::text_to_binary_file;
use crate::utils::get_compression_method;


pub fn add_files_directly_to_zip(
    zip: &Mutex<ZipWriter<File>>,
    folder_path: &Path,
    compression_algorithm: &str,
    compression_level: i64,
) -> io::Result<()> {
    let entries: Vec<_> = std::fs::read_dir(folder_path)?.filter_map(|e| e.ok()).collect();

    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();

            let (compression_method, valid_level) = match get_compression_method(compression_algorithm, compression_level) {
                Ok((method, level)) => (method, level),
                Err(e) => {
                    println!("Error getting compression method: {:?}", e);
                    return;
                }
            };

            let options: FileOptions<()> = FileOptions::default()
                .compression_method(compression_method).compression_level(Option::from(valid_level));

            let mut zip_guard = zip.lock().unwrap();
            match zip_guard.start_file(file_name, options) {
                Ok(_) => {
                    let mut file = match File::open(&path) {
                        Ok(file) => file,
                        Err(e) => {
                            println!("Error opening file: {:?}", e);
                            return;
                        }
                    };
                    if std::io::copy(&mut file, &mut *zip_guard).is_err() {
                        println!("Error adding file to zip: {}", file_name);
                    }
                },
                Err(e) => println!("Error starting file in zip: {}, {:?}", file_name, e),
            }
        } else {
            println!("Skipping non-file or directory: {:?}", path);
        }
    });

    Ok(())
}