use clap::Parser;
use delta::create_delta_file;
use file_ops::{open_read_handler, open_write_handler};
use opts::*;
use signature::create_signature_file;

mod delta;
mod file_ops;
mod opts;
mod rolling_sum;
mod signature;

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
