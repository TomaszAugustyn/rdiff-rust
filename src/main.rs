use clap::Parser;
use opts::*;
use std::fs::File;
use std::io::Result;
use std::path::Path;

mod opts;

fn main() {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Signature(s) => {
            println!("unchanged file: {}", s.unchanged_file.display());
            println!("signature file: {}", s.signature_file.display());
            let _unchanged_file = open_read_handler(&s.unchanged_file).unwrap();
            let _signature_file = open_write_handler(&s.signature_file).unwrap();
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
