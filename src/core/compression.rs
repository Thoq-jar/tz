use rayon::prelude::*;

pub fn compress(input: Vec<u8>) -> Vec<u8> {
    if input.len() < 10000 {
        return compress_sequential(input);
    }

    let chunk_size = std::cmp::max(input.len() / rayon::current_num_threads(), 1024);

    let chunks: Vec<&[u8]> = input.chunks(chunk_size).collect();

    let compressed_chunks: Vec<Vec<u8>> = chunks
        .par_iter()
        .map(|chunk| {
            let mut compressed = Vec::new();
            let mut i = 0;

            while i < chunk.len() {
                let byte = chunk[i];
                let mut count = 1;

                while i + 1 < chunk.len() && chunk[i + 1] == byte && count < 255 {
                    count += 1;
                    i += 1;
                }

                compressed.push(count);
                compressed.push(byte);
                i += 1;
            }

            compressed
        })
        .collect();

    let mut final_compressed = Vec::new();
    for chunk in compressed_chunks {
        final_compressed.extend(chunk);
    }

    final_compressed
}

fn compress_sequential(input: Vec<u8>) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let byte = input[i];
        let mut count = 1;

        while i + 1 < input.len() && input[i + 1] == byte && count < 255 {
            count += 1;
            i += 1;
        }

        compressed.push(count);
        compressed.push(byte);
        i += 1;
    }

    compressed
}
