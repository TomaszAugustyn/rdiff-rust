use super::rolling_sum::RollingSum;
use super::signature::{chunk_strong_hash, FileSignature};
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result};

#[derive(Debug, Serialize, Deserialize)]
pub enum Operation {
    Match(u32),
    NoMatch(Vec<u8>),
}

pub fn create_delta_file(
    sig_file: &File,
    modified_file: &File,
    delta_file: &mut File,
) -> Result<()> {
    let sig_reader = BufReader::new(sig_file);
    let signature: FileSignature = deserialize_from(sig_reader).unwrap();
    let chunk_size = signature.chunk_size as usize;
    let mut mod_file_reader = BufReader::new(modified_file);
    let mut buffer = super::read_file_to_buffer(&mut mod_file_reader)?;

    let delta = generate_delta(&mut buffer, &signature, chunk_size);

    let mut delta_writer = BufWriter::new(delta_file);
    serialize_into(&mut delta_writer, &delta).unwrap();

    Ok(())
}

pub fn generate_delta(
    buffer: &mut Vec<u8>,
    sig: &FileSignature,
    chunk_size: usize,
) -> Vec<Operation> {
    let mut operations: Vec<Operation> = Vec::new();
    loop {
        // In case less than whole chunk is left,
        // we have to narrow down the buffer to the leftover
        let chunk = &buffer[..cmp::min(chunk_size, buffer.len())];
        let chunk_len = chunk.len();
        if chunk_len == 0 {
            break;
        }

        // Calculate weak signature (using rsync rolling checksum algorithm) for chunk
        let mut rolling_sum = RollingSum::new();
        rolling_sum.update(chunk);
        let weak_hash = rolling_sum.digest();

        if let Some(hashes) = sig.chunk_hashes(&weak_hash) {
            let strong_hash = chunk_strong_hash(chunk);
            if let Some(hash) = hashes.into_iter().find(|h| h.strong_hash == strong_hash) {
                operations.push(Operation::Match(hash.chunk_index));

                // It was the last chunk
                if chunk_len < chunk_size || buffer.len() == chunk_size {
                    break;
                }
                // Prepare buffer for next iteration
                buffer.drain(..chunk_len);
                continue;
            }
        }
    }
    operations
}
