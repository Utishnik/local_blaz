use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::env;

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
    let src = text.as_bytes();
    let n = src.len();
    let key = secret_key.as_bytes();
    let key_len = key.len();
    let full_groups = n / GROUP_SIZE;
    let out_len = n + full_groups * MARKER.len();
    let mut middle = Vec::<u8>::with_capacity(out_len);

    let mut i = n;
    let mut ki = 0usize;
    while i > 0 {
        let start = if i >= GROUP_SIZE { i - GROUP_SIZE } else { 0 };
        let cnt = i - start;
        for r in (0..cnt).rev() {
            middle.push(src[start + r] ^ key[ki]);
            ki = (ki + 1) % key_len;
        }
        i = start;
        if cnt == GROUP_SIZE {
            middle.push(b'$' ^ key[ki]);
            ki = (ki + 1) % key_len;
            middle.push(b'$' ^ key[ki]);
            ki = (ki + 1) % key_len;
        }
    }

    BASE64.encode(&middle)
}

pub fn decode(encoded: &str, secret_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_bytes = BASE64.decode(encoded.trim().as_bytes())?;
    let decrypted = xor_bytes(&decoded_bytes, secret_key.as_bytes());
    let stripped = strip_markers(&decrypted, GROUP_SIZE, MARKER.len());
    let original_bytes = reverse_bytes(&stripped);
    String::from_utf8(original_bytes).map_err(Into::into)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <encode|decode> <key> <text>", args[0]);
        std::process::exit(1);
    }
    let mode = &args[1];
    let key = &args[2];
    let text = &args[3];
    if mode == "decode" {
        match decode(text, key) {
            Ok(r) => println!("{}", r),
            Err(e) => {
                eprintln!("Decode error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", encode(text, key));
    }
}
