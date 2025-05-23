use std::io;
use zip::CompressionMethod;

/// Determines the `CompressionMethod` and validates the compression level based on the algorithm name.
///
/// This function takes a string slice representing the compression algorithm (e.g., "Zstd", "Bzip2", "Deflated")
/// and a compression level. It returns a tuple containing the corresponding `CompressionMethod` enum variant
/// and an `Option<i64>` for the validated compression level. If the provided level is outside the valid range
/// for the algorithm, a default level is used.
///
/// # Arguments
///
/// * `algorithm` - A string slice representing the desired compression algorithm.
/// * `level` - The compression level to use. Valid ranges depend on the algorithm.
///
/// # Returns
///
/// An `io::Result` containing a tuple of `(CompressionMethod, Option<i64>)` on success, or an `io::Error`
/// if the specified algorithm is unsupported.
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

