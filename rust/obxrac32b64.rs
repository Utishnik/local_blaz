use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::error::Error;
use std::io::{self, Write};
use std::time::Instant;

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
    BASE64.encode(encrypted)
}

pub fn decode(encoded: &str, secret_key: &str) -> Result<String, Box<dyn Error>> {
    let decoded_bytes = BASE64.decode(encoded.trim().as_bytes())?;
    let decrypted = xor_bytes(&decoded_bytes, secret_key.as_bytes());
    let stripped = strip_markers(&decrypted, GROUP_SIZE, MARKER.len());
    let original_bytes = reverse_bytes(&stripped);
    String::from_utf8(original_bytes).map_err(Into::into)
}

fn prompt(label: &str) -> Result<String, io::Error> {
    print!("{}: ", label);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim_end_matches(['\r', '\n']).to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("obxrac32b64 cryptor");
    let mode = prompt("mode (e - encode / d - decode)")?;
    let key = prompt("secret_key")?;
    let text = prompt("text")?;

    let start = Instant::now();
    if mode.starts_with('d') {
        match decode(&text, &key) {
            Ok(decrypted) => println!("Decrypted: {}", decrypted),
            Err(e) => eprintln!("Decode error: {}", e),
        }
    } else {
        println!("Encrypted: {}", encode(&text, &key));
    }
    println!("Elapsed: {:0.6}s", start.elapsed().as_secs_f64());

    Ok(())
}
