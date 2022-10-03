use std::num::Wrapping;
use rand::{self, RngCore};

fn rotl(x: Wrapping<u64>, k: usize) -> Wrapping<u64> {
	return (x << k) | (x >> (64 - k));
}

///    This is xoshiro256++ 1.0, one of our all-purpose, rock-solid generators.
///    It has excellent (sub-ns) speed, a state (256 bits) that is large
///    enough for any parallel application, and it passes all tests we are
///    aware of.
///
///    For generating just floating-point numbers, xoshiro256+ is even faster.
///
///    The state must be seeded so that it is not everywhere zero. If you have
///    a 64-bit seed, we suggest to seed a splitmix64 generator and use its
///    output to fill s.
pub struct Xoshiro256 {
    state: [Wrapping<u64>; 4],
}

impl Xoshiro256 {
    pub fn from_entropy() -> Self {
        let mut state = [Wrapping(0u64); 4];
        for i in 0..4 {
            state[i].0 = rand::rngs::OsRng.next_u64();
        }
        Self { state }
    }

    /// Get the next random number in a sequence
    pub fn next(&mut self) -> u64 {
        let s = &mut self.state;

        let result = rotl(s[0] + s[3], 23) + s[0];

        let t = s[1] << 17;

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];

        s[2] ^= t;

        s[3] = rotl(s[3], 45);

        return result.0;
    }

    /// This is the jump function for the generator. It is equivalent
    /// to 2^128 calls to next(); it can be used to generate 2^128
    /// non-overlapping subsequences for parallel computations.
    pub fn jump(&mut self) {
        const JUMP: [u64; 4] = [ 0x180ec6d33cfd0aba, 0xd5a61266f0c9392c, 0xa9582618e03fc9aa, 0x39abdc4529b1661c ];

        let mut s0 = Wrapping(0u64);
        let mut s1 = Wrapping(0u64);
        let mut s2 = Wrapping(0u64);
        let mut s3 = Wrapping(0u64);
        for i in 0..4 {
            for b in 0..64u32 {
                if (JUMP[i] & 1u64 << b) > 0 {
                    s0 ^= self.state[0];
                    s1 ^= self.state[1];
                    s2 ^= self.state[2];
                    s3 ^= self.state[3];
                }
                self.next();	
            }
        }
            
        self.state[0] = s0;
        self.state[1] = s1;
        self.state[2] = s2;
        self.state[3] = s3;
    }

    /// This is the long-jump function for the generator. It is equivalent to
    /// 2^192 calls to next(); it can be used to generate 2^64 starting points,
    /// from each of which jump() will generate 2^64 non-overlapping
    /// subsequences for parallel distributed computations.
    pub fn long_jump(&mut self) {
        const LONG_JUMP: [u64; 4] = [ 0x76e15d3efefdcbbf, 0xc5004e441c522fb3, 0x77710069854ee241, 0x39109bb02acbe635 ];

        let mut s0 = Wrapping(0u64);
        let mut s1 = Wrapping(0u64);
        let mut s2 = Wrapping(0u64);
        let mut s3 = Wrapping(0u64);
        for i in 0..4 {
            for b in 0..64u32 {
                if (LONG_JUMP[i] & 1u64 << b) > 0 {
                    s0 ^= self.state[0];
                    s1 ^= self.state[1];
                    s2 ^= self.state[2];
                    s3 ^= self.state[3];
                }
                self.next();	
            }
        }
            
        self.state[0] = s0;
        self.state[1] = s1;
        self.state[2] = s2;
        self.state[3] = s3;
    }
}
