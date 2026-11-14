//! Bit-exact port of `numpy.random.default_rng(seed).permutation(n)` (numpy 2.x: `SeedSequence` →
//! `PCG64` (XSL-RR 128/64) → `Generator.shuffle` = Fisher–Yates with `random_interval`).
//!
//! Frozen splits made by the Python freezer used this generator; reproducing it bit for bit keeps
//! every committed split id valid under the Rust freezer (same seed, same rule text, same groups).

const INIT_A: u32 = 0x43b0_d7e5;
const MULT_A: u32 = 0x931e_8875;
const INIT_B: u32 = 0x8b51_f9dd;
const MULT_B: u32 = 0x58f3_8ded;
const MIX_MULT_L: u32 = 0xca01_f9dd;
const MIX_MULT_R: u32 = 0x4973_f715;
const XSHIFT: u32 = 16;
const POOL: usize = 4;

fn hashmix(mut value: u32, hash_const: &mut u32) -> u32 {
    value ^= *hash_const;
    *hash_const = hash_const.wrapping_mul(MULT_A);
    value = value.wrapping_mul(*hash_const);
    value ^ (value >> XSHIFT)
}

fn mix(x: u32, y: u32) -> u32 {
    let r = MIX_MULT_L
        .wrapping_mul(x)
        .wrapping_sub(MIX_MULT_R.wrapping_mul(y));
    r ^ (r >> XSHIFT)
}

/// `SeedSequence(entropy).generate_state(n_words, np.uint64)` for a non-negative integer entropy.
pub fn seed_sequence_u64(entropy: u128, n_words: usize) -> Vec<u64> {
    let mut ent: Vec<u32> = Vec::new();
    let mut e = entropy;
    loop {
        ent.push((e & 0xffff_ffff) as u32);
        e >>= 32;
        if e == 0 {
            break;
        }
    }
    let mut pool = [0u32; POOL];
    let mut hc = INIT_A;
    for (i, slot) in pool.iter_mut().enumerate() {
        *slot = hashmix(ent.get(i).copied().unwrap_or(0), &mut hc);
    }
    for i_src in 0..POOL {
        for i_dst in 0..POOL {
            if i_src != i_dst {
                let h = hashmix(pool[i_src], &mut hc);
                pool[i_dst] = mix(pool[i_dst], h);
            }
        }
    }
    for &v in ent.iter().skip(POOL) {
        for i_dst in 0..POOL {
            let h = hashmix(v, &mut hc);
            pool[i_dst] = mix(pool[i_dst], h);
        }
    }
    let mut hb = INIT_B;
    let words32: Vec<u32> = (0..n_words * 2)
        .map(|i| {
            let mut d = pool[i % POOL];
            d ^= hb;
            hb = hb.wrapping_mul(MULT_B);
            d = d.wrapping_mul(hb);
            d ^ (d >> XSHIFT)
        })
        .collect();
    words32
        .chunks(2)
        .map(|c| c[0] as u64 | ((c[1] as u64) << 32))
        .collect()
}

const PCG_MULT: u128 = 0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645;

pub struct Pcg64 {
    state: u128,
    inc: u128,
    has_u32: bool,
    u32_buf: u32,
}

impl Pcg64 {
    /// `np.random.default_rng(seed)` bit generator.
    pub fn from_seed(seed: u64) -> Pcg64 {
        let s = seed_sequence_u64(seed as u128, 4);
        let initstate = ((s[0] as u128) << 64) | s[1] as u128;
        let initseq = ((s[2] as u128) << 64) | s[3] as u128;
        let mut g = Pcg64 {
            state: 0,
            inc: (initseq << 1) | 1,
            has_u32: false,
            u32_buf: 0,
        };
        g.step();
        g.state = g.state.wrapping_add(initstate);
        g.step();
        g
    }
    fn step(&mut self) {
        self.state = self.state.wrapping_mul(PCG_MULT).wrapping_add(self.inc);
    }
    pub fn next_u64(&mut self) -> u64 {
        self.step();
        let s = self.state;
        let x = ((s >> 64) as u64) ^ (s as u64);
        x.rotate_right((s >> 122) as u32)
    }
    pub fn next_u32(&mut self) -> u32 {
        if self.has_u32 {
            self.has_u32 = false;
            return self.u32_buf;
        }
        let n = self.next_u64();
        self.has_u32 = true;
        self.u32_buf = (n >> 32) as u32;
        n as u32
    }
    /// numpy `random_interval(bitgen, max)`: uniform on [0, max] by masked rejection.
    pub fn interval(&mut self, max: u64) -> u64 {
        if max == 0 {
            return 0;
        }
        let mut mask = max;
        for s in [1, 2, 4, 8, 16, 32] {
            mask |= mask >> s;
        }
        if max <= 0xffff_ffff {
            loop {
                let v = (self.next_u32() as u64) & mask;
                if v <= max {
                    return v;
                }
            }
        }
        loop {
            let v = self.next_u64() & mask;
            if v <= max {
                return v;
            }
        }
    }
}

/// `np.random.default_rng(seed).permutation(n)`.
pub fn permutation(seed: u64, n: usize) -> Vec<usize> {
    let mut g = Pcg64::from_seed(seed);
    let mut x: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = g.interval(i as u64) as usize;
        x.swap(i, j);
    }
    x
}
