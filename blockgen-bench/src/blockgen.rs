use std::vec::Vec;
// use aes::Aes128;
// use aes::cipher::{
//     BlockEncrypt, KeyInit,
//     generic_array::GenericArray,
// };
use blake3;

pub const E: usize = 13;
pub const D: usize = E + 1; // Amount of dependencies of each fragment
pub const N: usize = 1 << E as usize; // Amount of fragment per block
const INDEX_MASK: usize = N - 1;

pub const ABSOLUTE_SPEEDUP_UPPERBOUND: usize = E*(1 << (E - 1))*E + (N/2) - 1;
const MIN_ADVERSARY_OPERATIONS: usize = 8000000; // 20GHz (adversary max speed) * 0.2ms (disk latency) * 2 (security margin)
// const OPERATIONS_PER_STEP: usize = 40;  // AES-128 is ~40 primitive operations
const OPERATIONS_PER_STEP: usize = 196/2;  // Blake3 is at least 196 sequential primitive operations per 64 bytes of input.
const MIN_ADVERSARY_STEPS: usize = (MIN_ADVERSARY_OPERATIONS + (OPERATIONS_PER_STEP - 1)) / OPERATIONS_PER_STEP;
const STEPS_LOWERBOUND: usize = MIN_ADVERSARY_STEPS + ABSOLUTE_SPEEDUP_UPPERBOUND;
pub const SIZE: usize = (STEPS_LOWERBOUND + (E - 1)) / E;
pub const STEPS: usize = SIZE * E;

pub const FRAGMENT_BYTES: usize = 32; // Cannot be more than 32 Bytes as the intermediate states of Blake uses 32 Bytes.
pub type Fragment = [u8; FRAGMENT_BYTES];
pub const BLOCK_BYTE_SIZE: usize = N * FRAGMENT_BYTES;

pub const GROUP_SIZE: usize = 1;
pub const GROUP_BYTE_SIZE: usize = BLOCK_BYTE_SIZE * GROUP_SIZE;
//pub type FragmentGroup = [Fragment; GROUP_SIZE];
pub type FragmentGroup = Fragment;
// pub type Block = [Fragment; N as usize];
pub type BlockGroup = Vec<FragmentGroup>;

pub const INIT_SIZE_EXP: usize = 2;
pub const INIT_SIZE: usize = 1 << INIT_SIZE_EXP; // 4x64bit = 256bit
pub const INIT_MASK: usize = INIT_SIZE - 1;
pub type InitGroup = [FragmentGroup; INIT_SIZE];

pub fn block_gen(inits: InitGroup) -> BlockGroup {
    if is_x86_feature_detected!("avx2") {
        unsafe { block_gen_avx2(inits) }
    } else {
        block_gen_inner(inits)
    }
}

#[target_feature(enable = "avx2")]
unsafe fn block_gen_avx2(inits: InitGroup) -> BlockGroup {
    block_gen_inner(inits)
}

#[inline(always)]
fn block_gen_inner(inits: InitGroup) -> BlockGroup {
    // let mut block: Block = [[0; FRAGMENTSIZE]; N as usize];
    let mut block: BlockGroup = vec![[0u8; FRAGMENT_BYTES]; N as usize];
    for i in 0..N {
        block[i] = inits[i & INIT_MASK];
    }
    
    let start = N - (SIZE % N);

    // let mut last: FragmentGroup = block[(start.wrapping_sub(1)) & INDEX_MASK];
    for i in 0..SIZE {
        let index = (i + start) % N;

        let mut hasher = blake3::Hasher::new();
        for j in 0..D {
            let jump = 1 << j;
            let target = (index + N - jump) & INDEX_MASK;
            let x = block[target];
            hasher.update(&x);
        }
        hasher.finalize_xof().fill(&mut block[index]);

        // let mut next_fragment = last;
        // for j in 1..D {
        //     let jump = 1 << j;
        //     // let target = (index + N - jump) % N;
        //     let target = index.wrapping_sub(jump) & INDEX_MASK;
        //     let x = block[target];
        //     // if (j & 1) == 1 {
        //     //     next_fragment = add(next_fragment, x);
        //     // } else {
        //     //     next_fragment = xor(next_fragment, x);
        //     // }
        //     next_fragment = aes(next_fragment, x);
        // }
        // //next_fragment = rot(next_fragment);
        // last = next_fragment;
        // block[index] = last;
    }
    return block;
}

// #[inline(always)]
// fn xoradd(a: FragmentGroup, b: FragmentGroup) -> FragmentGroup {
//     let c: FragmentGroup = xor(a, b);
//     return add(c, a)
// }

// #[inline(always)]
// fn aes(a: FragmentGroup, b: FragmentGroup) -> FragmentGroup {
//     let mut c: FragmentGroup = [0u8; FRAGMENT_BYTES];
//     c[..].copy_from_slice(&a);
//     let cipher = Aes128::new(&GenericArray::from(b));
//     cipher.encrypt_block(&mut GenericArray::from(c));
//     return c
// }

// #[inline(always)]
// fn add(a: FragmentGroup, b: FragmentGroup) -> FragmentGroup {
//     let mut c: FragmentGroup = [0; GROUP_SIZE];
//     for i in 0..GROUP_SIZE {
//         c[i] = a[i].wrapping_add(b[i]);
//     }
//     return c
// }

// #[inline(always)]
// fn xor(a: FragmentGroup, b: FragmentGroup) -> FragmentGroup {
//     let mut c: FragmentGroup = [0; GROUP_SIZE];
//     for i in 0..GROUP_SIZE {
//         c[i] = a[i] ^ b[i];
//     }
//     return c
// }

// #[inline(always)]
// fn rot(a: FragmentGroup) -> FragmentGroup {
//     let mut b: FragmentGroup = [0; GROUP_SIZE];
//     for i in 0..GROUP_SIZE {
//         b[i] = a[i].rotate_left(19); // Recommended shifts : 11/19/27
//     }
//     return b;
// }