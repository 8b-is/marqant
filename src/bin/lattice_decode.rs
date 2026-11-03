//! Decoder-only control for the preceding integer lattice experiment.
use std::{env, fs, io};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("Usage: lattice_decode INPUT OUTPUT MAX_OUTPUT_BYTES".into());
    }
    let bound = args[3].parse::<usize>()?;
    let encoded = fs::read(&args[1])?;
    let decoded = marqant::mem8_lattice::decompress(&encoded, bound)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;
    fs::write(&args[2], decoded)?;
    Ok(())
}
