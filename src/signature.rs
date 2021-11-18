use super::rolling_sum::chunk_rollsum;
use bincode::serialize_into;
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};

/// Default block size in rsync C implementation
const BLOCK_SIZE: u32 = 700;
/// Hasher output length for strong signature (Blake2) need to be 32 bytes (256 bits)
/// to comply with rsync C implementation
const RS_MAX_STRONG_SUM_LENGTH: usize = 32;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSignature {
    /// Chunk size used to calculate weak and strong signatures
    pub chunk_size: u32,
    /// Key is a weak signature (rsync rolling checksum algorithm)
    /// Value is a vector of all strong hashes together with the index
    /// of their chunk for which weak signature is the same
    pub signature_table: HashMap<u32, Vec<ChunkHash>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkHash {
    pub chunk_index: u32,
    pub strong_hash: [u8; RS_MAX_STRONG_SUM_LENGTH],
}

impl FileSignature {
    pub fn chunk_hashes(&self, key: &u32) -> Option<&Vec<ChunkHash>> {
        self.signature_table.get(key)
    }
}

pub fn create_signature_file(input_file: &File, sig_file: &mut File) -> Result<()> {
    // Fallback set to BLOCK_SIZE
    let chunk_size = input_file
        .metadata()
        .map_or(BLOCK_SIZE, |meta| calculate_chunk_size(meta.len()));

    let mut input_reader = BufReader::new(input_file);
    let mut buffer = super::read_file_to_buffer(&mut input_reader)?;

    let signature = generate_signature(&mut buffer, chunk_size);

    // Write serialized signature to file
    let mut sig_writer = BufWriter::new(sig_file);
    serialize_into(&mut sig_writer, &signature).unwrap();
    Ok(())
}

pub fn generate_signature(buffer: &mut Vec<u8>, chunk_size: u32) -> FileSignature {
    let mut signature = FileSignature {
        chunk_size,
        signature_table: HashMap::new(),
    };

    let chunk_size = chunk_size as usize;
    let mut chunk_index = 0u32;

    loop {
        // In case less than whole chunk is left,
        // we have to narrow down the buffer to the leftover
        let chunk = &buffer[..cmp::min(chunk_size, buffer.len())];
        let chunk_len = chunk.len();
        if chunk_len == 0 {
            break;
        }

        // Calculate weak signature (using rsync rolling checksum algorithm) for chunk
        let weak_hash = chunk_rollsum(chunk);

        // Calculate strong signature (using Blake2b).
        let strong_hash = chunk_strong_hash(chunk);

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
        buffer.drain(..chunk_len);
        chunk_index += 1;
    }
    signature
}

pub fn chunk_strong_hash(chunk: &[u8]) -> [u8; RS_MAX_STRONG_SUM_LENGTH] {
    // Use blake2 as MD5 is cryptographically broken:
    // https://www.kb.cert.org/vuls/id/836068
    let mut hasher = VarBlake2b::new(RS_MAX_STRONG_SUM_LENGTH).unwrap();
    hasher.update(&chunk);
    let mut strong_hash = [0u8; RS_MAX_STRONG_SUM_LENGTH];
    hasher.finalize_variable(|res| {
        strong_hash = res.try_into().expect("slice with incorrect length");
    });
    strong_hash
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
