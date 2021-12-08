use clap::Parser;
use delta::create_delta_file;
use opts::*;
use signature::create_signature_file;
use std::fs::File;
use std::io::Result;
use std::io::{BufReader, Read};
use std::path::Path;

mod delta;
mod opts;
pub mod rolling_sum;
pub mod signature;

fn main() {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Signature(s) => {
            println!("unchanged file: {}", s.unchanged_file.display());
            println!("signature file: {}", s.signature_file.display());
            let unchanged_file = open_read_handler(&s.unchanged_file).unwrap();
            let mut signature_file = open_write_handler(&s.signature_file).unwrap();
            create_signature_file(&unchanged_file, &mut signature_file).unwrap();
        }
        SubCommand::Delta(d) => {
            println!("signature file: {}", d.signature_file.display());
            println!("modified file: {}", d.modified_file.display());
            println!("delta file: {}", d.delta_file.display());
            let signature_file = open_read_handler(&d.signature_file).unwrap();
            let modified_file = open_read_handler(&d.modified_file).unwrap();
            let mut delta_file = open_write_handler(&d.delta_file).unwrap();
            create_delta_file(&signature_file, &modified_file, &mut delta_file).unwrap();
        }
    }
}

fn read_file_to_buffer(reader: &mut BufReader<&File>) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn open_read_handler(input_path: &Path) -> Result<File> {
    match File::open(input_path) {
        Ok(file) => Ok(file),
        Err(err) => {
            eprintln!(
                "cannot open file for reading: {:?}, error: {}",
                input_path, err
            );
            Err(err)
        }
    }
}

fn open_write_handler(output_path: &Path) -> Result<File> {
    match File::create(output_path) {
        Ok(file) => Ok(file),
        Err(err) => {
            eprintln!(
                "cannot open file for writing: {:?}, error: {}",
                output_path, err
            );
            Err(err)
        }
    }
}
