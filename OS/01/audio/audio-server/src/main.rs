mod transcribe;

use clap::Parser;
use std::path::PathBuf;
use transcribe::transcribe;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// This is the model for Whisper STT
    #[arg(short, long, value_parser, required = true)]
    model_path: PathBuf,
    
    /// This is the wav audio file that will be converted from speech to text
    #[arg(short, long, value_parser, required = true)]
    file_path: PathBuf,
}

fn main() {

    let args = Args::parse();

    println!("Model: {}", args.model_path.display());
    println!("File: {}", args.file_path.display());

    let result = transcribe(&args.model_path, &args.file_path);

    match result {
        Ok(transcription) => println!("Transcription:\n{}", transcription),
        Err(e) => println!("Error: {}", e),
    }
}
