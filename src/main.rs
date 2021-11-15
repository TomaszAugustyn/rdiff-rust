use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use clap::Parser;
use opts::*;
use rolling_sum::*;
use std::cmp;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::path::Path;

mod opts;
mod rolling_sum;

/// Default block size in rsync C implementation
const BLOCK_SIZE: u32 = 700;
/// Hasher output length for strong signature (Blake2) need to be 32 bytes (256 bits)
/// to comply with rsync C implementation
const RS_MAX_STRONG_SUM_LENGTH: usize = 32;

struct FileSignature {
    /// Chunk size used to calculate weak and strong signatures
    chunk_size: u32,
    /// Key is a weak signature (rsync rolling checksum algorithm)
    /// Value is a vector of all strong hashes together with the index
    /// of their chunk for which weak signature is the same
    signature_table: HashMap<u32, Vec<ChunkHash>>,
}

struct ChunkHash {
    chunk_index: u32,
    strong_hash: [u8; RS_MAX_STRONG_SUM_LENGTH],
}

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

    let mut signature = FileSignature {
        chunk_size,
        signature_table: HashMap::new(),
    };

    let chunk_size = chunk_size as usize;
    let mut input_reader = BufReader::new(input_file);
    let buffer = read_file_to_buffer(&mut input_reader)?;
    let mut rolling_sum = RollingSum::new();
    let mut chunk_index = 0u32;

    loop {
        // In case less than whole chunk is left,
        // we have to narrow down the buffer to the leftover
        let chunk = &buffer[..cmp::min(chunk_size, buffer.len())];
        let chunk_len = chunk.len();
        if chunk_len == 0 {
            break;
        }

        // Calculate weak signature (using rsync rolling checksum algorithm) from chunk
        rolling_sum.update(chunk);
        let weak_hash = rolling_sum.digest();

        // Calculate strong signature. Use blake2 as MD5 is cryptographically broken:
        // https://www.kb.cert.org/vuls/id/836068
        let mut hasher = VarBlake2b::new(RS_MAX_STRONG_SUM_LENGTH).unwrap();
        hasher.update(&chunk);
        let mut strong_hash = [0u8; RS_MAX_STRONG_SUM_LENGTH];
        hasher.finalize_variable(|res| {
            strong_hash = res.try_into().expect("slice with incorrect length");
        });

        // Add entry to signature table
        let chunk_hashes = signature
            .signature_table
            .entry(weak_hash)
            .or_insert_with(Vec::new);

        chunk_hashes.push(ChunkHash {
            chunk_index,
            strong_hash,
        });

        // It was the last chunk
        if chunk_len < chunk_size || buffer.len() == chunk_size {
            break;
        }
        // Prepare buffer for next iteration
        let buffer = &buffer[chunk_len..];
        chunk_index += 1;
    }

    // TODO: write signature to file
    //let mut sig_writer = BufWriter::new(sig_file);

    Ok(())
}

fn write_u32(writer: &mut BufWriter<&mut File>, value: u32) -> Result<usize> {
    writer.write(&value.to_be_bytes())
}

fn read_file_to_buffer(reader: &mut BufReader<&File>) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn calculate_chunk_size(file_length: u64) -> u32 {
    // According to the rsync source code:
    // https://git.samba.org/?p=rsync.git;a=blob;f=generator.c;hb=ca538965d81290ebd514397916594bdb2857e378#l690
    // block size is calculated by rounding (to the multiple of 8) square root of the file length if it is
    // greater than BLOCK_SIZE * BLOCK_SIZE (490 000 bytes) otherwise it is BLOCK_SIZE (currently set to 700 bytes)
    if file_length <= (BLOCK_SIZE * BLOCK_SIZE) as u64 {
        BLOCK_SIZE
    } else {
        (((file_length as f64).sqrt() / 8.0).round() * 8.0) as u32
    }
}
