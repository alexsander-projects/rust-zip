# Blazingly fast Rust compressor and decompressor

This app is capable of compressing and decompressing files using either the Zstd, Bzip2 or deflate algorithms.
It is written in Rust and uses the `zip` crate for the compression and decompression.

Optionally, you can specify if you want to convert the files to binary before compressing them,
this will result in a smaller file size, which can be useful for text files.

Note that the decompression process is capable of converting the files back to their original format.

The app can convert text, json, and images to binary. It can also convert binary files back to their original format.

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

The Struct zip::write::FileOptions provides the compression method and level to use when writing a file to a ZIP archive.

The compression method can be one of the following(with their respective compression levels):


    Deflated:10 - 264 for Zopfli, 0 - 9 for other encoders. Default is 24 if Zopfli is the only encoder, or 6 otherwise.
    Bzip2: 0 - 9. Default is 6
    Zstd: -7 - 22, with zero being mapped to default level. Default is 3



## Usage

### To compress a file, run the following command:

```bash
cargo run -- compression <input_folder> <output_zip> <compression_algorithm> <compression_level> [--convert_to_binary]
```

Where:
- `<input_folder>` is the path to the folder you want to compress
- `<output_zip>` is the path to the output zip file
- `<compression_algorithm>` is the compression algorithm to use.
  It can be either `zstd`, `bzip2` or `deflate`
- `<compression_level>` is the compression level to use. Depending on the algorithm,
  it can be a number between -7 and 22 for Zstd, 0 and 9 for Bzip2, and 0 and 9 for Deflate.
> Note: Higher compression levels can result in reduced file size but will take longer to compress.
- `--convert_to_binary` is an optional flag that will convert the files to binary before compressing them.

### To decompress a file, run the following command:

```bash
cargo run -- decompression <zip_path> <output_folder> [--decompress_without_conversion]
```

Where:
- `<zip_path>` is the path to the zip file you want to decompress
- `<output_folder>` is the path to the output folder
- `--decompress_without_conversion` is an optional flag that will decompress
  the files without converting them back to their original format.

## Performance

- The compression and decompression speed where roughly 10 times faster than 7zip for the Zstd algorithm,
  where text files were compressed. The file size was roughly the same.
- Roughly 10 times faster for the Zstd algorithm, where image files were compressed.
- Roughly 3 times faster for the Zstd algorithm, where json files were compressed.

## Conversion to binary performance

- Converting to binary format has almost no impact on the compression speed, when converting text files.