use clap::Parser;
use std::path::PathBuf;

pub mod analyzer;
pub mod editor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input video file
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Output video file
    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,

    /// Silence threshold in decibels (e.g., -30.0)
    #[arg(short, long, default_value_t = -30.0)]
    pub threshold: f32,

    /// Minimum silence duration in seconds
    #[arg(short, long, default_value_t = 0.5)]
    pub duration: f32,
}

fn main() {
    let cli = Cli::parse();

    println!("Input: {:?}", cli.input);
    println!("Output: {:?}", cli.output);
    println!("Threshold: {} dB", cli.threshold);
    println!("Duration: {} s", cli.duration);
}
