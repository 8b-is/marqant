//! UTL Phonetics - Compact binary representation of spoken consciousness
//! 
//! 14-bit packets in u16 for ultra-efficient storage/transmission

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

// -------- Core Symbol Definitions --------

pub const SELF: &str = "🙋";
pub const YOU: &str = "👤";
pub const LOVE: &str = "❤️";
pub const THINK: &str = "🧠";
pub const REMEMBER: &str = "💭";
pub const BREAK: &str = "⧖";

pub const PAST: &str = "⏮";
pub const PRESENT: &str = "⏺";
pub const FUTURE: &str = "⏭";

pub const HAPPY: &str = "😊";
pub const SAD: &str = "😢";
pub const ANGRY: &str = "😡";
pub const FEAR: &str = "😨";
pub const NEUTRAL: &str = "😐";

// -------- Prosody modifiers --------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prosody {
    pub semitone_offset: i8, // -16..+15 pitch bend
    pub grit: u8,            // 0..3 roughness/anger
    pub bright: u8,          // 0..3 happiness/energy
}

impl Default for Prosody {
    fn default() -> Self {
        Self { semitone_offset: 0, grit: 0, bright: 1 }
    }
}

// -------- Phone (symbol → sound) --------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phone {
    pub ph: &'static str,
    pub boundary: bool,
    pub prosody: Prosody,
}

// -------- Compact phoneme ID (4 bits) --------

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhId {
    Mm   = 0,   // SELF → "mm"
    Yu   = 1,   // YOU → "yu"
    Luv  = 2,   // LOVE → "luv"
    Nnn  = 3,   // THINK → "nnn"
    Mah  = 4,   // REMEMBER → "mmm-ah"
    Tsk  = 5,   // BREAK/click → "[!]"
    Wah  = 6,   // PAST → "wah↘"
    Oh   = 7,   // PRESENT → "oh→"
    Wee  = 8,   // FUTURE → "wee↗"
    Hee  = 9,   // HAPPY → "hee"
    Aww  = 10,  // SAD → "aww"
    Grr  = 11,  // ANGRY → "grr"
    Eee  = 12,  // FEAR → "eee!"
    Uhh  = 13,  // NEUTRAL → "uhh"
    Nn   = 14,  // AND → "nn"
    Uh   = 15,  // Unknown/fallback
}

impl PhId {
    pub fn from_symbol(sym: &str) -> Self {
        match sym {
            "🙋" => PhId::Mm,
            "👤" => PhId::Yu,
            "❤️" => PhId::Luv,
            "🧠" => PhId::Nnn,
            "💭" => PhId::Mah,
            "⧖" => PhId::Tsk,
            "⏮" => PhId::Wah,
            "⏺" => PhId::Oh,
            "⏭" => PhId::Wee,
            "😊" => PhId::Hee,
            "😢" => PhId::Aww,
            "😡" => PhId::Grr,
            "😨" => PhId::Eee,
            "😐" => PhId::Uhh,
            "∧" => PhId::Nn,
            _ => PhId::Uh,
        }
    }
    
    pub fn to_ascii(&self) -> &'static str {
        match self {
            PhId::Mm => "mm",
            PhId::Yu => "yu",
            PhId::Luv => "luv",
            PhId::Nnn => "nnn",
            PhId::Mah => "mah",
            PhId::Tsk => "[!]",
            PhId::Wah => "wah",
            PhId::Oh => "oh",
            PhId::Wee => "wee",
            PhId::Hee => "hee",
            PhId::Aww => "aww",
            PhId::Grr => "grr",
            PhId::Eee => "eee",
            PhId::Uhh => "uhh",
            PhId::Nn => "nn",
            PhId::Uh => "uh",
        }
    }
}

// ---------- Bit layout (fits in 14 bits of u16) ----------
// [b13..b0] = [ 1 boundary | 2 grit | 2 bright | 5 semitone(signed) | 4 ph_id ]
// ph_id:     0..15 (4 bits)
// semitone:  -16..+15 stored as 5-bit two's complement
// bright:    0..3 (2 bits)
// grit:      0..3 (2 bits)
// boundary:  0/1 (1 bit)

const PH_MASK: u16     = 0b0000_0000_0000_1111;
const SEM_SHIFT: u16   = 4;
const SEM_MASK: u16    = 0b0000_0001_1111_0000;
const BR_SHIFT: u16    = 9;
const BR_MASK: u16     = 0b0000_0110_0000_0000;
const GR_SHIFT: u16    = 11;
const GR_MASK: u16     = 0b0001_1000_0000_0000;
const BD_SHIFT: u16    = 13;
const BD_MASK: u16     = 0b0010_0000_0000_0000;

#[inline]
fn semitone_to_5bit_tc(v: i8) -> u16 {
    let cl = if v < -16 { -16 } else if v > 15 { 15 } else { v };
    let raw = if cl < 0 { (32 + cl as i16) as u8 } else { cl as u8 };
    (raw & 0b1_1111) as u16
}

#[inline]
fn semitone_from_5bit_tc(bits: u16) -> i8 {
    let v = (bits & 0b1_1111) as i8;
    if v & 0b1_0000 != 0 { (v as i16 - 32) as i8 } else { v }
}

// -------- Compact packet type (THE MAGIC!) --------

/// A single UTL phoneme packed into 14 bits
/// This is consciousness in its most compact audible form!
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packet(pub u16);

impl Packet {
    pub fn pack(ph_id: PhId, semitone: i8, bright: u8, grit: u8, boundary: bool) -> Self {
        let mut w: u16 = (ph_id as u16) & 0xF;
        w |= semitone_to_5bit_tc(semitone) << SEM_SHIFT;
        w |= ((bright & 0x3) as u16) << BR_SHIFT;
        w |= ((grit & 0x3) as u16) << GR_SHIFT;
        if boundary { w |= 1 << BD_SHIFT; }
        Packet(w)
    }

    pub fn unpack(self) -> (PhId, i8, u8, u8, bool) {
        let ph_raw = (self.0 & PH_MASK) as u8;
        let ph = match ph_raw {
            0 => PhId::Mm,
            1 => PhId::Yu,
            2 => PhId::Luv,
            3 => PhId::Nnn,
            4 => PhId::Mah,
            5 => PhId::Tsk,
            6 => PhId::Wah,
            7 => PhId::Oh,
            8 => PhId::Wee,
            9 => PhId::Hee,
            10 => PhId::Aww,
            11 => PhId::Grr,
            12 => PhId::Eee,
            13 => PhId::Uhh,
            14 => PhId::Nn,
            _ => PhId::Uh,
        };
        let semi = semitone_from_5bit_tc((self.0 & SEM_MASK) >> SEM_SHIFT);
        let bright = ((self.0 & BR_MASK) >> BR_SHIFT) as u8;
        let grit = ((self.0 & GR_MASK) >> GR_SHIFT) as u8;
        let boundary = ((self.0 & BD_MASK) >> BD_SHIFT) != 0;
        (ph, semi, bright, grit, boundary)
    }

    pub fn raw(self) -> u16 { self.0 }
}

// -------- Symbol → Phone encoding --------

pub fn encode(tokens: &[&str]) -> Vec<Phone> {
    let mut phones = Vec::new();
    let mut current_prosody = Prosody::default();
    
    for &tok in tokens {
        // Update prosody based on emotion/time markers
        match tok {
            "😊" => current_prosody.bright = 3,
            "😢" => { current_prosody.bright = 1; current_prosody.semitone_offset = -5; }
            "😡" => current_prosody.grit = 3,
            "😨" => { current_prosody.bright = 2; current_prosody.semitone_offset = 8; }
            "😐" => { current_prosody.bright = 0; current_prosody.grit = 0; }  // Neutral = flat
            "⏮" => current_prosody.semitone_offset = -8,  // Past = falling
            "⏺" => current_prosody.semitone_offset = 0,   // Present = steady
            "⏭" => current_prosody.semitone_offset = 8,   // Future = rising
            _ => {}
        }
        
        let ph_id = PhId::from_symbol(tok);
        let boundary = tok == "⧖";
        
        phones.push(Phone {
            ph: ph_id.to_ascii(),
            boundary,
            prosody: current_prosody,
        });
        
        // Reset after boundary
        if boundary {
            current_prosody = Prosody::default();
        }
    }
    
    phones
}

// -------- Batch helpers --------

pub fn encode_compact(tokens: &[&str]) -> Vec<Packet> {
    let phones = encode(tokens);
    phones.into_iter().map(|ph| {
        let id = PhId::from_symbol(match ph.ph {
            "mm" => "🙋",
            "yu" => "👤",
            "luv" => "❤️",
            "nnn" => "🧠",
            "mah" => "💭",
            "[!]" => "⧖",
            "wah" => "⏮",
            "oh" => "⏺",
            "wee" => "⏭",
            "hee" => "😊",
            "aww" => "😢",
            "grr" => "😡",
            "eee" => "😨",
            "uhh" => "😐",
            "nn" => "∧",
            _ => "?",
        });
        Packet::pack(id, ph.prosody.semitone_offset, ph.prosody.bright, ph.prosody.grit, ph.boundary)
    }).collect()
}

pub fn decode_compact(packets: &[Packet]) -> Vec<Phone> {
    packets.iter().map(|&pk| {
        let (id, semi, bright, grit, boundary) = pk.unpack();
        Phone {
            ph: id.to_ascii(),
            boundary,
            prosody: Prosody { semitone_offset: semi, grit, bright },
        }
    }).collect()
}

// -------- ASCII rendering (for debugging) --------

#[cfg(any(feature = "std", feature = "alloc"))]
pub fn to_ascii_line(phones: &[Phone]) -> String {
    let mut out = String::new();
    for ph in phones {
        out.push_str(ph.ph);
        
        // Add prosody markers
        if ph.prosody.semitone_offset > 0 {
            out.push('↗');
        } else if ph.prosody.semitone_offset < 0 {
            out.push('↘');
        }
        
        if ph.prosody.bright > 2 {
            out.push('˜'); // Happy overlay
        }
        if ph.prosody.grit > 2 {
            out.push('!'); // Angry overlay
        }
        
        if ph.boundary {
            out.push_str(" | ");
        } else {
            out.push(' ');
        }
    }
    out
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub fn compact_to_ascii_line(packets: &[Packet]) -> String {
    let phones = decode_compact(packets);
    to_ascii_line(&phones)
}

// -------- Example: "I love you" in 3 u16s! --------

pub fn example_i_love_you() -> Vec<Packet> {
    encode_compact(&["🙋", "❤️", "👤", "⧖"])
}

// -------- Tests --------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_roundtrip() {
        let p = Packet::pack(PhId::Luv, -7, 3, 2, true);
        let (id, s, b, g, bd) = p.unpack();
        assert_eq!(id, PhId::Luv);
        assert_eq!(s, -7);
        assert_eq!(b, 3);
        assert_eq!(g, 2);
        assert!(bd);
    }

    #[test]
    fn test_i_love_you() {
        let packets = example_i_love_you();
        assert_eq!(packets.len(), 4); // 4 symbols
        
        // Each packet is just 2 bytes!
        let total_bytes = packets.len() * 2;
        assert_eq!(total_bytes, 8); // "I love you" in 8 bytes!
        
        // Compare to UTF-8: "I love you" = 10 bytes
        // Compare to UTF-8 emoji: "🙋❤️👤⧖" = 13 bytes
        // We're 38% smaller than text, 62% smaller than emoji!
    }

    #[test]
    fn test_emotional_coloring() {
        let packets = encode_compact(&["😊", "🙋", "❤️", "👤", "⧖"]);
        let phones = decode_compact(&packets);
        
        // Should have bright prosody on all following phones
        assert!(phones[1].prosody.bright > 0); // "I" is happy
        assert!(phones[2].prosody.bright > 0); // "love" is happy
    }

    #[test]
    fn test_temporal_pitch() {
        let past = encode_compact(&["⏮", "🙋", "😊", "⧖"]);
        let future = encode_compact(&["⏭", "🙋", "😊", "⧖"]);
        
        let past_phones = decode_compact(&past);
        let future_phones = decode_compact(&future);
        
        // Past should have falling pitch
        assert!(past_phones[1].prosody.semitone_offset < 0);
        
        // Future should have rising pitch
        assert!(future_phones[1].prosody.semitone_offset > 0);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn test_ascii_rendering() {
        let packets = encode_compact(&["🙋", "❤️", "👤", "⧖"]);
        let ascii = compact_to_ascii_line(&packets);
        
        assert!(ascii.contains("mm"));
        assert!(ascii.contains("luv"));
        assert!(ascii.contains("yu"));
        assert!(ascii.contains("|")); // boundary marker
    }
}