use clap::Parser;
use opts::*;
use std::fs::File;
use std::io::Result;
use std::path::Path;

mod opts;
mod rolling_sum;

const BLOCK_SIZE: u64 = 700;

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
        }
    }
}

fn open_read_handler(input_path: &Path) -> Result<Box<File>> {
    match File::open(input_path) {
        Ok(file) => Ok(Box::new(file)),
        Err(err) => {
            eprintln!(
                "cannot open file for reading: {:?}, error: {}",
                input_path, err
            );
            Err(err)
        }
    }
}

fn open_write_handler(output_path: &Path) -> Result<Box<File>> {
    match File::create(output_path) {
        Ok(file) => Ok(Box::new(file)),
        Err(err) => {
            eprintln!(
                "cannot open file for writing: {:?}, error: {}",
                output_path, err
            );
            Err(err)
        }
    }
}

fn create_signature_file(input_file: &File, sig_file: &mut File) -> Result<()> {
    // Fallback set to BLOCK_SIZE
    let chunk_size = input_file
        .metadata()
        .map_or(BLOCK_SIZE, |meta| calculate_chunk_size(meta.len()));
    let mut buffer = vec![0; chunk_size as usize];

    Ok(())
}

fn calculate_chunk_size(file_length: u64) -> u64 {
    // According to the rsync source code:
    // https://git.samba.org/?p=rsync.git;a=blob;f=generator.c;hb=ca538965d81290ebd514397916594bdb2857e378#l690
    // block size is calculated by rounding (to the multiple of 8) square root of the file length if it is
    // greater than BLOCK_SIZE * BLOCK_SIZE (490 000 bytes) otherwise it is BLOCK_SIZE (currently set to 700 bytes)
    if file_length <= BLOCK_SIZE * BLOCK_SIZE {
        BLOCK_SIZE
    } else {
        (((file_length as f64).sqrt() / 8.0).round() * 8.0) as u64
    }
}
