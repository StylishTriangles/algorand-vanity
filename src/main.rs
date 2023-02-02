mod cpu;
mod mnemonic;
mod wordlist;
mod crypto;
mod gpu;
mod xoshiro256;

use crate::cpu::run as run_cpu;
use crate::gpu::run as run_gpu;
use std::env;

fn main() {
    let input: Vec<String> = env::args().collect();
    if input.len() != 3 {
        println!("Usage: {} [--cpu|--gpu] PREFIX", input[0]);
        return;
    }
    let mode = input[1].clone();
    let prefix = input[2].clone();
    match mode.as_str() {
        "--cpu" => run_cpu(prefix, 8),
        "--gpu" => run_gpu(prefix).unwrap(),
        _ => println!("Unknown mode: {}", mode)
    }
}
