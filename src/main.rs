mod run;
mod xoshiro256;

use crate::run::run;
use std::env;

fn main() {
    let input: Vec<String> = env::args().collect();
    if input.len() != 2 {
        println!("Usage: {} PREFIX", input[0]);
        return;
    }
    match run(input[1].clone()) {
        Ok(_) => {}
        Err(e) => println!("Program encountered an error: {:?}", e)
    };
}
