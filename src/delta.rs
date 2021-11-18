use super::rolling_sum::RollingSum;
use super::signature::{chunk_strong_hash, is_chunk_last, ChunkHash, FileSignature};
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};

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
        let mut chunk_len = chunk.len();
        if chunk_len == 0 {
            break;
        }

        // Calculate weak signature (using rsync rolling checksum algorithm) for chunk
        let mut rolling_sum = RollingSum::new();
        rolling_sum.update(chunk);
        let weak_hash = rolling_sum.digest();

        if let Some(hash) = chunk_hash_matching_weak_n_strong(sig, weak_hash, chunk) {
            operations.push(Operation::Match(hash.chunk_index));

            if is_chunk_last(chunk_len, buffer.len()) {
                break;
            }
            // Prepare buffer for next iteration
            buffer.drain(..chunk_len);
            continue;
        }

        let mut not_matching_bytes: Vec<u8> = Vec::new();
        loop {
            let mut buf_len = buffer.len();
            let mut next: Option<u8> = None;
            if !is_chunk_last(chunk_len, buf_len) {
                next = Some(buffer[chunk_size]);
            }
            if buf_len > 0 {
                let prev = buffer.remove(0);
                buf_len = buffer.len();
                not_matching_bytes.push(prev);
                rolling_sum.roll_fw(prev, next);
                let weak_hash = rolling_sum.digest();
                let chunk = &buffer[..cmp::min(chunk_size, buf_len)];
                chunk_len = chunk.len();

                if let Some(hash) = chunk_hash_matching_weak_n_strong(sig, weak_hash, chunk) {
                    operations.push(Operation::NoMatch(not_matching_bytes));
                    operations.push(Operation::Match(hash.chunk_index));
                    // Prepare buffer for next iteration
                    buffer.drain(..chunk_len);
                    break;
                }
            } else {
                if !not_matching_bytes.is_empty() {
                    operations.push(Operation::NoMatch(not_matching_bytes));
                }
                break;
            }
        }
    }
    operations
}

fn chunk_hash_matching_weak_n_strong<'a>(
    sig: &'a FileSignature,
    weak_hash: u32,
    chunk: &[u8],
) -> Option<&'a ChunkHash> {
    if let Some(hashes) = sig.chunk_hashes(&weak_hash) {
        let strong_hash = chunk_strong_hash(chunk);
        hashes.iter().find(|h| h.strong_hash == strong_hash)
    } else {
        None
    }
}
