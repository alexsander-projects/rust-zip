# Blazingly fast Rust compressor and decompressor

A Rust application for compressing and decompressing files using Zstd, Bzip2, or Deflate algorithms via the `zip` crate.
Features optional binary conversion for text, JSON, and image files to reduce size, with the ability to revert during decompression.

## Table of Contents

- [Notes](#notes)
  - [Tokio](#tokio)
  - [Rayon](#rayon)
  - [Zip](#zip)
- [Usage](#usage)
  - [Compressing Files](#compressing-files)
  - [Decompressing Files](#decompressing-files)
- [Performance](#performance)
- [Conversion to binary performance](#conversion-to-binary-performance)

## Notes

### Tokio:

In the decompression process, Tokio is used to manage asynchronous file and directory operations efficiently.
This is particularly useful for non-blocking I/O operations, such as reading from or writing to files,
which are common in file decompression tasks.
Using Tokio allows the application to perform these operations without blocking the execution of other parts of the program,
improving overall performance and responsiveness,
especially when dealing with large files or a large number of files.

### Rayon:

Rayon library is utilized to parallelize the process of adding files to a ZIP archive.
This is achieved by iterating
over the entries in a directory and processing each file concurrently, rather than sequentially.
This can significantly reduce the time taken to compress a large number of files, as each file can be compressed in parallel,
taking advantage of multicore processors.

### Zip:

According to the official documentation, that can be found [here](https://docs.rs/zip/2.1.3/zip/index.html):

A library for reading and writing ZIP archives.
ZIP is a format designed for cross-platform file “archiving”.
That is, storing a collection of files in a single datastream to make them easier to share between computers.
Additionally, ZIP is able to compress and encrypt files in its archives.

The `zip::write::FileOptions` struct provides the compression method and level to use when writing a file to a ZIP archive.

The compression method can be one of the following (with their respective compression levels):

*   **Deflated**: Levels 0-9 (standard encoders) or 10-264 (Zopfli). Default: 6 (standard) or 24 (if Zopfli is the only encoder).
*   **Bzip2**: Levels 0-9. Default: 6.
*   **Zstd**: Levels -7 to 22 (0 maps to default level 3). Default: 3.

## Usage

### Compressing Files

To compress a file, run the following command:

```bash
cargo run -- compression <input_folder> <output_zip> [--compression-algorithm <algorithm>] [--compression-level <level>] [--convert_to_binary]
```
Or using short flags:
```bash
cargo run -- compression <input_folder> <output_zip> [-c <algorithm>] [-l <level>] [--convert_to_binary]
```

Where:
- `<input_folder>` is the path to the folder you want to compress
- `<output_zip>` is the path to the output zip file
- `<algorithm>` is the compression algorithm to use (e.g., `Zstd`, `Bzip2`, `Deflate`). This should follow the `--compression-algorithm` or `-c` flag. Defaults to `Zstd`.
- `<level>` is the compression level to use. This should follow the `--compression-level` or `-l` flag. Depending on the algorithm,
  it can be a number between -7 and 22 for Zstd, 0 and 9 for Bzip2, and 0 and 9 for Deflate. (lower numbers mean faster compression). Defaults to `3` for Zstd.
> Note: Higher compression levels can result in reduced file size but will take longer to compress.
- `--convert_to_binary` is an optional flag that will convert the files to binary before compressing them.

Example:
```bash
cargo run -- compression ./my_folder ./archive.zip --compression-algorithm Zstd --compression-level 3
```
Or with short flags and default algorithm/level:
```bash
cargo run -- compression ./my_folder ./archive.zip 
```
Or with short flags:
```bash
cargo run -- compression ./my_folder ./archive.zip -c Zstd -l 3
```

Example with binary conversion:
```bash
cargo run -- compression ./my_folder ./archive.zip -c Zstd -l 3 --convert_to_binary
```

### Decompressing Files

To decompress a file, run the following command:

```bash
cargo run -- decompression <zip_path> <output_folder> [--decompress_without_conversion]
```

Where:
- `<zip_path>` is the path to the zip file you want to decompress
- `<output_folder>` is the path to the output folder
- `--decompress_without_conversion` is an optional flag that will decompress
  the files without converting them back to their original format.

Example:
```bash
cargo run -- decompression ./archive.zip ./output_directory
```

Example without conversion:
```bash
cargo run -- decompression ./archive.zip ./output_directory --decompress_without_conversion
```

## Performance

- The compression and decompression speed where roughly 10 times faster than 7zip for the Zstd algorithm,
  where text files were compressed. The file size was roughly the same.
- Roughly 10 times faster for the Zstd algorithm, where image files were compressed.
- Roughly 3 times faster for the Zstd algorithm, where json files were compressed.

## Conversion to binary performance

- Converting to binary format has almost no impact on the compression speed, when converting text files.

