# Blazingly fast Rust compressor and decompressor

This app is capable of compressing and decompressing files using either the Zstd, Bzip2 or deflate algorithms. 
It is written in Rust and uses the `zip` crate for the compression and decompression.

Optionally, you can specify if you want to convert the files to binary before compressing them,
this will result in a smaller file size, which can be useful for text files.

If chosen, note that the decompression process will also convert the files back to their original format.

## Usage

### To compress a file, run the following command:

```bash
cargo run -- compression <input_folder> <output_zip> <compression_algorithm> <compression_level>
```

Where:
- `<input_folder>` is the path to the folder you want to compress
- `<output_zip>` is the path to the output zip file
- `<compression_algorithm>` is the compression algorithm to use.
It can be either `zstd`, `bzip2` or `deflate`
- `<compression_level>` is the compression level to use. Depending on the algorithm,
it can be a number between -7 and 22 for Zstd, 0 and 9 for Bzip2, and 0 and 9 for Deflate.
> Note: Higher compression levels can result in reduced file size but will take longer to compress.

### To decompress a file, run the following command:

```bash
cargo run -- decompression <zip_path> <output_folder>
```

Where:
- `<zip_path>` is the path to the zip file you want to decompress
- `<output_folder>` is the path to the output folder

## Performance

- The compression and decompression speed where roughly 3 times faster than 7zip for the Zstd algorithm,
where text files were compressed. The file size was roughly the same.