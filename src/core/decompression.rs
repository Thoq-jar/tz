use rayon::prelude::*;
use std::io;

pub fn decompress(compressed: Vec<u8>) -> Result<String, std::string::FromUtf8Error> {
    let bytes = decompress_binary(compressed).unwrap();
    String::from_utf8(bytes)
}

pub fn decompress_binary(compressed: Vec<u8>) -> io::Result<Vec<u8>> {
    if compressed.len() < 10000 {
        return Ok(decompress_sequential_binary(compressed));
    }

    let mut work_items = Vec::new();
    let mut i = 0;

    while i + 1 < compressed.len() {
        let count = compressed[i] as usize;
        let byte = compressed[i + 1];
        work_items.push((count, byte));
        i += 2;
    }

    let decompressed_chunks: Vec<Vec<u8>> = work_items
        .par_iter()
        .map(|(count, byte)| {
            let mut chunk = Vec::with_capacity(*count);
            for _ in 0..*count {
                chunk.push(*byte);
            }
            chunk
        })
        .collect();

    let mut final_decompressed = Vec::new();
    for chunk in decompressed_chunks {
        final_decompressed.extend(chunk);
    }

    Ok(final_decompressed)
}

fn decompress_sequential_binary(compressed: Vec<u8>) -> Vec<u8> {
    let mut decompressed = Vec::new();
    let mut i = 0;

    while i + 1 < compressed.len() {
        let count = compressed[i] as usize;
        let byte = compressed[i + 1];

        for _ in 0..count {
            decompressed.push(byte);
        }

        i += 2;
    }

    decompressed
}

fn decompress_sequential(compressed: Vec<u8>) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(decompress_sequential_binary(compressed))
}
