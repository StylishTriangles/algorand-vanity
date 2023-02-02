
use ed25519_dalek::{SecretKey, PublicKey};
use sha2::Digest;


const CHECKSUM_LEN: usize = 32;
const PUBLIC_KEY_LEN: usize = 32;
#[allow(dead_code)]
const SEED_LEN: usize = 32;


#[derive(Copy,Clone)]
pub struct KeyPair {
    pub secret_key: [u8; 32],
    pub public_key: [u8; 32],
}

impl KeyPair {
    pub fn from_seed(seed: [u8; 32]) -> KeyPair {
        let secret_key = SecretKey::from_bytes(&seed).unwrap();
        let public_key = PublicKey::from(&secret_key);
        KeyPair {
            secret_key: secret_key.to_bytes(),
            public_key: public_key.to_bytes(),
        }
    }

    /// Works similarily to an address, but skips calculating the checksum
    /// Returns true if the address starts with the given prefix
    pub fn check_prefix(&self, prefix: &str) -> bool {
        let encoded = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &self.public_key);
        encoded.starts_with(prefix)
    }

    pub fn address(&self) -> String {
        let mut address: Vec<u8> = Vec::with_capacity(CHECKSUM_LEN + PUBLIC_KEY_LEN);
        address.extend(&self.public_key);
        let checksum = sha2::Sha512_256::digest(&address)[28..32].to_vec();
        address.extend(checksum);
        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &address)
    }
}