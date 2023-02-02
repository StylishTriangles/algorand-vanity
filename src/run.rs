use std::{fs};
use ocl::{ProQue, Buffer, MemFlags};
// use sha2::{Sha512, Digest};

// use ring::signature::{Ed25519KeyPair};
use crate::xoshiro256::Xoshiro256;

use base32::{Alphabet, encode as b32encode};

const DIMS: usize = 10;
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

// pub struct Ed25519KeyPairDebug {
//     pub private_scalar: [u8; 32],

//     pub private_prefix: [u8; 32],

//     pub public_key: [u8; 32],
// }


pub(crate) fn run(prefix: String) -> Result<(), ocl::Error> {
    let mut rng = Xoshiro256::from_entropy();
    let src =  load_kernel();

    let pro_que = ProQue::builder()
        .src(src)
        .dims(DIMS)
        .build()?;


    let found_buffer = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(DIMS)
        .flags(MemFlags::WRITE_ONLY)
        .build()?;

    let pk_buffer = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(PUBLIC_KEY_LEN * DIMS)
        .flags(MemFlags::WRITE_ONLY)
        .fill_val(0u8)
        .build()?;

    // pro_que.create_buffer()
    let seeds = generate_seeds(&mut rng, DIMS);
    // let mut seeds: Vec<u8> = vec![108, 75, 154, 1, 135, 158, 88, 246, 92, 77, 139, 103, 47, 229, 239, 40, 220, 185, 84, 75, 117, 203, 247, 26, 91, 7, 240, 156, 134, 212, 162, 234];
    let seed_buffer;
    unsafe {
        seed_buffer = Buffer::<u8>::builder()
            .queue(pro_que.queue().clone())
            .len(seeds.len())
            .use_host_slice(&seeds)
            .flags(MemFlags::READ_ONLY)
            .build()?;
    };
    
    let prefix_buffer = Buffer::<u8>::builder()
        .queue(pro_que.queue().clone())
        .len(prefix.len())
        .copy_host_slice(prefix.as_bytes())
        .flags(MemFlags::READ_ONLY)
        .build()?;

    let kernel = pro_que.kernel_builder("brute_force_b32_prefix")
        .arg(&found_buffer)
        .arg(&pk_buffer)
        .arg(&seed_buffer)
        .arg(&prefix_buffer)
        .arg(prefix.len() as u32)
        .arg(1u32)
        .build()?;

    unsafe { kernel.enq()?; }

    let mut public_keys: Vec<u8> = vec![0u8; pk_buffer.len()];
    let mut found: Vec<u8> = vec![0u8; DIMS];
    found_buffer.read(&mut found).enq()?;
    pk_buffer.read(&mut public_keys).enq()?;

    // let acc = Account::from_seed(seeds[0..32].try_into().unwrap());
    // seeds.reverse();
    // let kp = Ed25519KeyPair::from_seed_unchecked(&seeds[0..32]).unwrap();
    // let kp_debug: Ed25519KeyPairDebug = unsafe {
    //     std::mem::transmute(kp)
    // };

    println!("Seeds: {:?}", seeds);
    println!("Found: {:?}", found);

    for i in 0..found.len() {
        // println!("Public key: {:?}", acc.raw_public_key());
        if true {
            let pk = &public_keys[i * PUBLIC_KEY_LEN..(i + 1) * PUBLIC_KEY_LEN];
            let pk_b32 = b32encode(Alphabet::RFC4648 { padding: false }, pk);
            println!("Found matching address: {}", pk_b32);
        }
        // public_keys = public_keys[32..].to_vec();
    }

    // println!("Public key: {:?}", kp_debug.public_key);
    // println!("Public key base32: {:?}", b32encode(Alphabet::RFC4648 { padding: true }, &kp_debug.public_key));
    // let sha = Sha512::digest(&seeds);
    // println!("SHA512: {:?}", sha);

    // println!("The value at index [{}] is now '{:?}'!", 0, &public_keys[0..32]);
    Ok(())
}