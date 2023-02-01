use crate::{xoshiro256::Xoshiro256};
use std::sync::mpsc;
use ed25519_dalek::{SecretKey, PublicKey};
use crate::mnemonic;
use sha2::Digest;

use base32::{Alphabet, encode as b32encode};

pub fn public_checksum(data: &[u8]) -> Vec<u8> {
    sha2::Sha512_256::digest(data)[28..32].to_vec()
}


pub fn convert(data: &[u64; 4]) -> [u8; 32] {
    unsafe { std::mem::transmute(*data) }
}

struct ThreadResult {
    pub address: String,
    pub mnemonic: String,
}

pub fn run(prefix: String, threads: usize) -> () {
    let mut main_rng = Xoshiro256::from_entropy();

    let (tx, rx) = mpsc::channel();
    for _ in 0..threads {
        main_rng.jump();
        let prefix = prefix.clone();
        let initial_seed = [main_rng.next(), main_rng.next(), main_rng.next(), main_rng.next()];
        let tx = tx.clone();
        std::thread::spawn(move || {
            run_cpu_thread(prefix, initial_seed, tx)
        });
    }
    let result = rx.recv().unwrap();
    println!("Match found!");
    println!("Address: {}", result.address);
    println!("Mnemonic: {}", result.mnemonic);
}

/// Search for a matching address in a single thread
/// Each thread must receive a different seed
fn run_cpu_thread(prefix: String, mut seed: [u64; 4], tx: mpsc::Sender<ThreadResult>) -> () {
    loop {
        seed[3] = seed[3].wrapping_add(1);

        let seed_bytes = convert(&seed);
        let sk = SecretKey::from_bytes(&seed_bytes).unwrap();
        let pk = PublicKey::from(&sk);
        let raw_pk = pk.as_bytes();
        let b32_pk = b32encode(Alphabet::RFC4648 { padding: false }, raw_pk);
        if b32_pk.starts_with(&prefix) {
            // the checksum is not added initially to avoid unnecessary computation,
            // however it's needed to display the full Algorand address
            let checksum = public_checksum(raw_pk);
            let pk_with_checksum = [raw_pk, checksum.as_slice()].concat();
            let b32_pk_ch = b32encode(Alphabet::RFC4648 { padding: false }, &pk_with_checksum);
            tx.send(ThreadResult {
                address: b32_pk_ch,
                mnemonic: mnemonic::from_key(&seed_bytes),
            }).unwrap();
            return;
        }
    }
}