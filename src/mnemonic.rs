/*
This file is a slight modification of the mnemonic module from the algonaut project.
*/


use sha2::Digest;
use crate::wordlist::WORDLIST;

const BITS_PER_WORD: usize = 11;
const MNEMONIC_DELIM: &str = " ";
type ChecksumAlg = sha2::Sha512_256;

// Assumes little-endian
fn to_u11_array(bytes: &[u8]) -> Vec<u32> {
    let mut buf = 0u32;
    let mut bit_count = 0;
    let mut out = Vec::with_capacity((bytes.len() * 8 + BITS_PER_WORD - 1) / BITS_PER_WORD);
    for &b in bytes {
        buf |= (u32::from(b)) << bit_count;
        bit_count += 8;
        if bit_count >= BITS_PER_WORD as u32 {
            out.push(buf & 0x7ff);
            buf >>= BITS_PER_WORD as u32;
            bit_count -= BITS_PER_WORD as u32;
        }
    }
    if bit_count != 0 {
        out.push(buf & 0x7ff);
    }
    out
}

// Returns a word corresponding to the 11 bit checksum of the data
fn checksum_word(data: &[u8; 32]) -> &str {
    let d = ChecksumAlg::digest(data);
    WORDLIST[to_u11_array(&d[0..2])[0] as usize]
}

/// Converts a 32-byte key into a 25 word mnemonic. The generated
/// mnemonic includes a checksum. Each word in the mnemonic represents 11 bits
/// of data, and the last 11 bits are reserved for the checksum.
pub fn from_key(key: &[u8; 32]) -> String {
    let check_word = checksum_word(key);
    let mut words: Vec<_> = to_u11_array(key).into_iter().map(| v | WORDLIST[v as usize] ).collect();
    words.push(check_word);
    words.join(MNEMONIC_DELIM)
}