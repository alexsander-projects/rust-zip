use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use tar::{Builder, Archive};
use zstd::stream::write::Encoder;
use std::io::Read;

#[derive(Parser)]
#[command(name = "targz_compressor")]
#[command(about = "A CLI tool to compress and decompress files using tar.gz with zstd compression", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compress {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long, default_value_t = 3)]
        level: i32,
    },
    Decompress {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compress { input, output, level } => {
            compress(&input, &output, *level)?;
        }
        Commands::Decompress { input, output } => {
            decompress(&input, &output)?;
        }
    }

    Ok(())
}

fn compress(input_path: &str, output_path: &str, level: i32) -> io::Result<()> {
    // Create a temporary tar file
    let tar_path = format!("{}.tar", output_path);
    let tar_file = File::create(&tar_path)?;
    let buf_writer = BufWriter::new(tar_file);

    // Create the tarball
    let mut tar = Builder::new(buf_writer);
    tar.append_dir_all(".", input_path)?;
    tar.finish()?;

    // Compress the tarball with Zstandard
    let tar_file = File::open(&tar_path)?;
    let tar_reader = BufReader::new(tar_file);
    let gz_file = File::create(output_path)?;
    let gz_writer = BufWriter::new(gz_file);

    println!("Using compression level: {}", level);

    let mut encoder = Encoder::new(gz_writer, level)?;
    encoder.multithread(num_cpus::get() as u32)?;

    io::copy(&mut tar_reader.take(u64::MAX), &mut encoder)?;
    encoder.finish()?;

    // Clean up the temporary tar file
    fs::remove_file(tar_path)?;

    Ok(())
}

fn decompress(input_path: &str, output_path: &str) -> io::Result<()> {
    let gz_file = File::open(input_path)?;
    let buf_reader = BufReader::new(gz_file);
    let decoder = zstd::stream::Decoder::new(buf_reader)?;
    let tar_reader = BufReader::new(decoder);

    let mut archive = Archive::new(tar_reader);
    archive.unpack(output_path)?;

    Ok(())
}
