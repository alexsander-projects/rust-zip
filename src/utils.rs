use std::io;
use zip::CompressionMethod;

pub fn get_compression_method(algorithm: &str, level: i64) -> io::Result<(CompressionMethod, Option<i64>)> {
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