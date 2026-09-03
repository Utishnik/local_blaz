use std::env;
use std::time::Instant;

use base64_simd::STANDARD as BASE64;

const GROUP_SIZE: usize = 5;
const MARKER: &[u8] = b"$$";

pub fn xor_bytes(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    data.iter()
        .zip(key.iter().cycle())
        .map(|(&b, &k)| b ^ k)
        .collect()
}

pub fn inject_markers(data: &[u8], group_size: usize, marker: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() + (data.len() / group_size) * marker.len());
    for chunk in data.chunks(group_size) {
        result.extend_from_slice(chunk);
        if chunk.len() == group_size {
            result.extend_from_slice(marker);
        }
    }
    result
}

pub fn strip_markers(data: &[u8], group_size: usize, marker_len: usize) -> Vec<u8> {
    let step = group_size + marker_len;
    data.chunks(step)
        .flat_map(|chunk| {
            let take = chunk.len().min(group_size);
            &chunk[..take]
        })
        .copied()
        .collect()
}

pub fn reverse_bytes(data: &[u8]) -> Vec<u8> {
    data.iter().rev().copied().collect()
}

pub fn encode(text: &str, secret_key: &str) -> String {
    let reversed = reverse_bytes(text.as_bytes());
    let marked = inject_markers(&reversed, GROUP_SIZE, MARKER);
    let encrypted = xor_bytes(&marked, secret_key.as_bytes());
    BASE64.encode_to_string(&encrypted)
}

pub fn decode(encoded: &str, secret_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_bytes = BASE64.decode_to_vec(encoded.trim().as_bytes())?;
    let decrypted = xor_bytes(&decoded_bytes, secret_key.as_bytes());
    let stripped = strip_markers(&decrypted, GROUP_SIZE, MARKER.len());
    let original_bytes = reverse_bytes(&stripped);
    String::from_utf8(original_bytes).map_err(Into::into)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <encode|decode> <key> <text> [iterations]",
            args[0]
        );
        std::process::exit(1);
    }

    let mode = &args[1];
    let key = &args[2];
    let text = &args[3];
    let iterations: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let is_decode = mode == "decode";

    let warmup = (iterations / 10).max(100);
    for _ in 0..warmup {
        if is_decode {
            let _ = decode(text, key);
        } else {
            let _ = encode(text, key);
        }
    }

    let start = Instant::now();
    let mut result = String::new();
    for _ in 0..iterations {
        if is_decode {
            match decode(text, key) {
                Ok(r) => result = r,
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            result = encode(text, key);
        }
    }
    let elapsed = start.elapsed();

    let total_ms = elapsed.as_secs_f64() * 1000.0;
    let avg_us = (total_ms * 1000.0) / iterations as f64;

    println!(
        "Rust-SIMD {} | iters={} | total={:.3}ms | avg={:.3}us/iter | result_len={}",
        mode, iterations, total_ms, avg_us, result.len()
    );
}
