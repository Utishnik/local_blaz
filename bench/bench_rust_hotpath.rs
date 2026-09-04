use std::env;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

const GROUP_SIZE: usize = 5;
const MARKER: &[u8] = b"$$";

#[hotpath::measure(label = "encode.reverse")]
pub fn step_reverse_bytes(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    out.extend(data.iter().rev().copied());
}

#[hotpath::measure(label = "encode.markers")]
pub fn step_inject_markers(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(data.len() + (data.len() / GROUP_SIZE) * MARKER.len());
    for chunk in data.chunks(GROUP_SIZE) {
        out.extend_from_slice(chunk);
        if chunk.len() == GROUP_SIZE {
            out.extend_from_slice(MARKER);
        }
    }
}

#[hotpath::measure(label = "encode.xor")]
pub fn step_xor(data: &[u8], key: &[u8], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(data.len());
    if key.is_empty() {
        out.extend_from_slice(data);
        return;
    }
    let mut ki = 0usize;
    for &b in data {
        out.push(b ^ key[ki]);
        ki += 1;
        if ki == key.len() {
            ki = 0;
        }
    }
}

#[hotpath::measure(label = "encode.base64")]
pub fn step_base64_encode(data: &[u8]) -> String {
    BASE64.encode(data)
}

#[hotpath::measure(label = "decode.base64")]
pub fn step_base64_decode(data: &[u8]) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(data)
}

#[hotpath::measure(label = "decode.strip")]
pub fn step_strip_markers(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    let step = GROUP_SIZE + MARKER.len();
    for chunk in data.chunks(step) {
        let take = chunk.len().min(GROUP_SIZE);
        out.extend_from_slice(&chunk[..take]);
    }
}

#[hotpath::measure(label = "encode.all")]
pub fn module_encode(text: &str, secret_key: &str) -> String {
    let mut reversed = Vec::with_capacity(text.len());
    step_reverse_bytes(text.as_bytes(), &mut reversed);

    let mut marked = Vec::with_capacity(reversed.len() + reversed.len() / GROUP_SIZE);
    step_inject_markers(&reversed, &mut marked);

    let mut xored = Vec::with_capacity(marked.len());
    step_xor(&marked, secret_key.as_bytes(), &mut xored);

    step_base64_encode(&xored)
}

#[hotpath::measure(label = "decode.all")]
pub fn module_decode(encoded: &str, secret_key: &str) -> Result<String, String> {
    let decoded_bytes = step_base64_decode(encoded.trim().as_bytes()).map_err(|e| e.to_string())?;

    let mut xored = Vec::with_capacity(decoded_bytes.len());
    step_xor(&decoded_bytes, secret_key.as_bytes(), &mut xored);

    let mut stripped = Vec::with_capacity(xored.len());
    step_strip_markers(&xored, &mut stripped);

    let mut reversed = Vec::with_capacity(stripped.len());
    step_reverse_bytes(&stripped, &mut reversed);

    String::from_utf8(reversed).map_err(|e| e.to_string())
}

pub fn encode(text: &str, secret_key: &str) -> String {
    module_encode(text, secret_key)
}

pub fn decode(encoded: &str, secret_key: &str) -> Result<String, String> {
    module_decode(encoded, secret_key)
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

#[hotpath::main]
fn main() {
    let args: Vec<String> = env::args().collect();

    let (mode, key, text, iterations, large);

    if args.len() >= 2 && args[1] == "--large" {
        let bytes: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        key = args.get(3).cloned().unwrap_or_default();
        mode = args.get(4).cloned().unwrap_or_else(|| "encode".into());
        iterations = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(3);
        large = true;
        text = gen_data(bytes);
    } else {
        key = args.get(2).cloned().unwrap_or_default();
        mode = args.get(1).cloned().unwrap_or_default();
        text = args.get(3).cloned().unwrap_or_default();
        iterations = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(10000);
        large = false;
    }

    let is_decode = mode == "decode";

    let warmup = if large { 0 } else { (iterations / 10).max(100) };
    for _ in 0..warmup {
        if is_decode {
            let _ = decode(&text, &key);
        } else {
            let _ = encode(&text, &key);
        }
    }

    hotpath::measure_block!("obxrac32b64_encode", {
        for _ in 0..iterations {
            let _ = encode(&text, &key);
        }
    });
    hotpath::measure_block!("obxrac32b64_decode", {
        for _ in 0..iterations {
            let _ = decode(&text, &key);
        }
    });

    println!("hotpath measurement done");
}
