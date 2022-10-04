use std::{fs};
use ocl::{ProQue, Buffer};
// use sha2::{Sha512, Digest};

use ring::signature::{Ed25519KeyPair};
use crate::xoshiro256::Xoshiro256;

const DIMS: usize = 1;
const PUBLIC_KEY_LEN: usize = 32;
const SEED_LEN: usize = 32;
const U64_LEN: usize = 8;


fn load_kernel() -> String {
    fs::read_to_string("src/chimera.cl").expect("Bruh, no such file")
}

pub fn generate_seeds(rng: &mut Xoshiro256, count: usize) -> Vec<u8> {
    let len: usize = (count * SEED_LEN / U64_LEN) as usize;
    let mut seeds = vec![0u64; len];
    for i in 0..len {
        seeds[i] = rng.next();
    }
    let (prefix, mid, suffix);
    unsafe {
        (prefix, mid, suffix) = seeds.align_to::<u8>();
    }
    assert!(prefix.is_empty());
    assert!(suffix.is_empty());
    return mid.to_vec();
}

pub struct Ed25519KeyPairDebug {
    pub private_scalar: [u8; 32],

    pub private_prefix: [u8; 32],

    pub public_key: [u8; 32],
}


pub(crate) fn run(prefix: String) -> Result<(), ocl::Error> {
    let mut rng = Xoshiro256::from_entropy();
    let src =  load_kernel();

    let pro_que = ProQue::builder()
        .src(src)
        .dims(DIMS)
        .build()?;


    let pk_buffer = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(PUBLIC_KEY_LEN * DIMS)
        .fill_val(0u8)
        .build()?;

    // pro_que.create_buffer()
    // let mut seeds = generate_seeds(&mut rng, DIMS);
    let mut seeds: Vec<u8> = vec![108, 75, 154, 1, 135, 158, 88, 246, 92, 77, 139, 103, 47, 229, 239, 40, 220, 185, 84, 75, 117, 203, 247, 26, 91, 7, 240, 156, 134, 212, 162, 234];
    println!("Seeds: {:?}", seeds);
    let seed_buffer = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(seeds.len())
        .copy_host_slice(&seeds)
        .build()?;

    let kernel = pro_que.kernel_builder("ed25519_create_pubkey")
        .arg(&pk_buffer)
        .arg(&seed_buffer)
        .build()?;

    unsafe { kernel.enq()?; }

    let mut vec: Vec<u8> = vec![0u8; pk_buffer.len()];
    pk_buffer.read(&mut vec).enq()?;

    // let acc = Account::from_seed(seeds[0..32].try_into().unwrap());
    // println!("Public key: {:?}", acc.raw_public_key());
    // seeds.reverse();
    let kp = Ed25519KeyPair::from_seed_unchecked(&seeds[0..32]).unwrap();
    let kp_debug: Ed25519KeyPairDebug = unsafe {
        std::mem::transmute(kp)
    };

    println!("Public key: {:?}", kp_debug.public_key);
    // let sha = Sha512::digest(&seeds);
    // println!("SHA512: {:?}", sha);

    println!("The value at index [{}] is now '{:?}'!", 0, &vec[0..32]);
    Ok(())
}