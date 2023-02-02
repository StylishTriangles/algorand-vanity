use crate::xoshiro256::Xoshiro256;
use std::sync::mpsc;
use crate::mnemonic;
use std::time::Instant;
use crate::crypto::KeyPair;
use std::io::stdout;
use std::io::Write;

/// Used for hashrate calculations
const HASH_MULTIPLIER: u64 = 1 << 14; // roughly 16k hashes
const HASH_MULTIPLIER_MASK: u64 = HASH_MULTIPLIER - 1;
/// How many seconds to remember for hashrate calculations
const REMEMBER_SECONDS: usize = 8;

pub fn convert(data: &[u64; 4]) -> [u8; 32] {
    unsafe { std::mem::transmute(*data) }
}

struct ThreadResult {
    pub address: String,
    pub mnemonic: String,
}

struct CircularBuffer {
    buffer: Vec<u64>,
    index: usize,
}

/// Used to measure the current key rate
impl CircularBuffer {
    pub fn new(size: usize) -> CircularBuffer {
        CircularBuffer {
            buffer: vec![0; size],
            index: 0,
        }
    }

    pub fn push(&mut self, value: u64) {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % self.buffer.len();
    }

    pub fn sum(&self) -> u64 {
        self.buffer.iter().sum()
    }
}

pub fn run(prefix: String, threads: usize) {
    let mut main_rng = Xoshiro256::from_entropy();

    let (tx, rx) = mpsc::channel();
    let (progress_sender, progress_receiver) = mpsc::channel();
    for _ in 0..threads {
        main_rng.jump();
        let prefix = prefix.clone();
        let initial_seed = [main_rng.next(), main_rng.next(), main_rng.next(), main_rng.next()];
        let tx = tx.clone();
        let progress_sender = progress_sender.clone();
        std::thread::spawn(move || {
            run_cpu_thread(prefix, initial_seed, tx, progress_sender)
        });
    }
    let mut recent_key_count = CircularBuffer::new(REMEMBER_SECONDS);
    let start = Instant::now();
    let mut last = start;
    println!("Search started!");
    loop {
        // Calculate the current hashrate
        let mut hash_count = 0;
        for _ in progress_receiver.try_iter() {
            hash_count += HASH_MULTIPLIER;
        }
        recent_key_count.push(hash_count);
        let total_recent_keys = recent_key_count.sum() as f64;
        let now = Instant::now();
        let time_elapsed = now.duration_since(last).as_millis() as f64;
        last = now;
        // display current hashrate
        let mut key_rate = total_recent_keys / time_elapsed / REMEMBER_SECONDS as f64;
        if key_rate.is_nan() {
            key_rate = 0.0;
        }
        print!("\rAvg. key search rate: {:.3}KK/s", key_rate);
        stdout().flush().unwrap();
        // check if the result is in
        if let Ok(result) = rx.try_recv() {
            println!();
            println!("Match found!");
            println!("Address: {}", result.address);
            println!("Mnemonic: {}", result.mnemonic);
            println!("Time: {}s", now.duration_since(start).as_secs());
            return;
        }
        // sleep for one second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Search for a matching address in a single thread
/// Each thread must receive a different seed
fn run_cpu_thread(prefix: String, mut seed: [u64; 4], tx: mpsc::Sender<ThreadResult>, progeress_sender: mpsc::Sender<()>) {
    let mut hashes = 0u64;
    loop {
        hashes += 1;
        if (hashes & HASH_MULTIPLIER_MASK) == 0 {
            progeress_sender.send(()).unwrap();
        }
        seed[3] = seed[3].wrapping_add(1);

        let kp = KeyPair::from_seed(convert(&seed));
        if kp.check_prefix(&prefix) {
            tx.send(ThreadResult {
                address: kp.address(),
                mnemonic: mnemonic::from_key(&kp.secret_key),
            }).unwrap();
            return;
        }
    }
}