//! Decoder-only development artifact for archive + decoder size comparisons.
//! No learning, encoder, or benchmark entry point is reachable from this binary.
use std::{env, fs, io};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("Usage: template_decode INPUT OUTPUT MAX_OUTPUT_BYTES".into());
    }
    let bound = args[3].parse::<usize>()?;
    let encoded = fs::read(&args[1])?;
    let decoded = marqant::salient_templates::decompress(&encoded, bound)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;
    fs::write(&args[2], decoded)?;
    Ok(())
}
