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
    let mut res = Vec::<u8>::new();
    res = reverse_bytes(text.as_bytes());
    res = inject_markers(&res, GROUP_SIZE, MARKER);
    res = xor_bytes(&res, secret_key.as_bytes());
    BASE64.encode_to_string(&res)
}

pub fn decode(encoded: &str, secret_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut res = Vec::<u8>::new();
    res = BASE64.decode_to_vec(encoded.trim().as_bytes())?;
    res = xor_bytes(&res, secret_key.as_bytes());
    res = strip_markers(&res, GROUP_SIZE, MARKER.len());
    res = reverse_bytes(&res);
    String::from_utf8(res).map_err(Into::into)
}

fn gen_data(bytes: usize) -> String {
    let pattern = "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = String::with_capacity(bytes);
    while out.len() < bytes {
        let take = std::cmp::min(bytes - out.len(), pattern.len());
        out.push_str(&pattern[..take]);
    }
    out
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (mode, key, text, iterations, large);

    if args.len() >= 2 && args[1] == "--large" {
        if args.len() < 4 {
            eprintln!(
                "Usage: {} --large <bytes> <key> [mode] [iterations]",
                args[0]
            );
            std::process::exit(1);
        }
        let bytes: usize = args[2].parse().unwrap_or(0);
        key = args[3].clone();
        mode = args.get(4).cloned().unwrap_or_else(|| "encode".into());
        iterations = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(3);
        large = true;
        text = gen_data(bytes);
    } else {
        if args.len() < 4 {
            eprintln!(
                "Usage: {} <encode|decode> <key> <text> [iterations]",
                args[0]
            );
            std::process::exit(1);
        }
        mode = args[1].clone();
        key = args[2].clone();
        text = args[3].clone();
        iterations = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1000);
        large = false;
    }

    let is_decode = mode == "decode";

    let mut bench_input = text;
    let mut encoded_input = String::new();
    if large && is_decode {
        encoded_input = encode(bench_input.as_str(), &key);
        bench_input = encoded_input.clone();
    }
    let input_len = bench_input.len();

    let warmup = if large { 1 } else { (iterations / 10).max(100) };
    for _ in 0..warmup {
        if is_decode {
            let _ = decode(&bench_input, &key);
        } else {
            let _ = encode(&bench_input, &key);
        }
    }

    let start = Instant::now();
    let mut result = String::new();
    for _ in 0..iterations {
        if is_decode {
            match decode(&bench_input, &key) {
                Ok(r) => result = r,
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            result = encode(&bench_input, &key);
        }
    }
    let elapsed = start.elapsed();

    let total_ms = elapsed.as_secs_f64() * 1000.0;
    let avg_us = (total_ms * 1000.0) / iterations as f64;
    let mbps = input_len as f64 / (total_ms / 1000.0) / (1024.0 * 1024.0);

    println!(
        "Rust-SIMD {} | input={}B | iters={} | total={:.3}ms | avg={:.3}us/iter | {:.2} MB/s | result_len={}",
        mode, input_len, iterations, total_ms, avg_us, mbps, result.len()
    );
}
