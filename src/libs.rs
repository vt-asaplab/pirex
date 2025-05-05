
#![allow(dead_code)]
#![allow(arithmetic_overflow)]
#![allow(non_camel_case_types)]

use aes::{Aes128, Block};
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use rand::rngs::OsRng;
use rand::RngCore;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::time::{Instant, Duration};
use memmap::{Mmap, MmapMut};
use packed_simd::{Simd, u8x64};
use bitvec::prelude::*;



pub const SERVER_ADDRESS: &str = "127.0.0.1:8111";

pub const BUNIT: usize = 4096; // one block has __ chunks
pub const USIZE: usize = 64; // one chunk has 64 bytes
pub const BSIZE: usize = BUNIT * USIZE; // block size

pub const NSIZE: usize = 4194304; // dbase size
pub const LSIZE: usize = 11; // logarithm sqrt
pub const SSIZE: usize = NSIZE >> LSIZE; // sqrt
pub const HSIZE: usize = SSIZE * 22; // hint size

pub const FRONT: u8 = 0b111;
pub const BFLAG: INDX = (SSIZE - 1) as INDX;

pub const KSIZE: usize = 0016; // one keyset has __ bytes
pub const ISIZE: usize = 0004; // one indice has __ bytes
pub const ISQRT: usize = 0002; // one offset has __ bytes

pub const IV_SIZE: usize = SSIZE * ISIZE;
pub const IV_SQRT: usize = SSIZE * ISQRT;

pub const THREAD: usize = 8;
pub const REGION: usize = HSIZE / THREAD;

pub type B_OFFSET = [u8; ISQRT]; // one offset as byte array
pub type B_INDICE = [u8; ISIZE]; // one indice as byte array
pub type B_KEYSET = [u8; KSIZE]; // one keyset as byte array
pub type B_UCHUNK = [u8; USIZE]; // one chunk as byte array

pub type LANE = u8x64;
pub type SIMD = Simd<B_UCHUNK>;
pub type SQRT = u16;
pub type INDX = u32;
pub type TSUB = (Vec<u8>, Vec<u8>);

pub const Z_OFFSET: B_OFFSET = [0; ISQRT];
pub const Z_INDICE: B_INDICE = [0; ISIZE];
pub const Z_KEYSET: B_KEYSET = [0; KSIZE];

pub const PSIZE : usize = HSIZE / 4; // parity selection bit len
pub const MSIZE : usize = 4; // number of element to group for modulo
pub const F_EXP : usize = 33;
pub const ESIZE : usize = BSIZE / MSIZE * F_EXP; // encrypted block size
pub type PINT = u32;


pub fn parse_index_1(_: INDX, bytes: & [u8]) -> usize
{
    let ppoint : B_OFFSET = bytes[.. ISQRT].try_into().unwrap();
    let mut offset : B_OFFSET = bytes[ISQRT ..].try_into().unwrap();
    
    offset[0] &= FRONT;

    let ppoint = SQRT::from_be_bytes(ppoint) as INDX;
    let offset = SQRT::from_be_bytes(offset) as INDX;
    let indice = (ppoint << LSIZE) + (offset);
    
    return indice as usize;
}

pub fn parse_index_2(pk: INDX, bytes: & [u8]) -> usize
{
    let mut offset : B_OFFSET = bytes.try_into().unwrap();

    offset[0] &= FRONT;

    let indice = pk + SQRT::from_be_bytes(offset) as INDX;
    
    return indice as usize;
}


pub struct Crypto {
    eval_block: [Block; SSIZE],
    prep_block: [B_OFFSET; SSIZE],
}

impl Crypto {

    pub fn new() -> Self
    {
        let block = GenericArray::from([0u8; 16]);
        let mut eval_block = [block; SSIZE];
        let mut prep_block = [Z_OFFSET; SSIZE];

        for i in 0 .. SSIZE
        {
            prep_block[i] = (i as SQRT).to_be_bytes();
            eval_block[i][.. ISQRT].copy_from_slice(& prep_block[i]);
        }
        
        Self {eval_block, prep_block}
    }

    pub fn os_random(& self, buffer: & mut [u8])
    {
        OsRng.fill_bytes(buffer);
    }

    pub fn ppr_val(& self, index: INDX) -> (usize, B_OFFSET)
    {
        let ppoint = (index >> LSIZE) as usize;
        let offset = ((index & BFLAG) as SQRT).to_be_bytes();

        return (ppoint, offset);
    }

    pub fn key_val(& self, key: & [u8], pk: usize) -> (B_OFFSET, Duration)
    {        
        let cipher = Aes128::new(key.into());

        let mut offset = Z_OFFSET;
        let mut block = self.eval_block[pk];

        let start = Instant::now();
        cipher.encrypt_block(& mut block);
        let finis = Instant::now();
        
        offset.copy_from_slice(& block[.. ISQRT]);

        offset[0] &= FRONT;

        return (offset, finis - start);
    }

    pub fn key_set(& self, key: & [u8]) -> Vec<u8>
    {
        let cipher = Aes128::new(key.into());

        let mut buffer = vec![0u8; IV_SQRT];
        let mut blocks = self.eval_block;
        cipher.encrypt_blocks(& mut blocks);

        for (i, offset) in buffer.chunks_exact_mut(ISQRT).enumerate()
        {
            offset.copy_from_slice(& blocks[i][.. ISQRT])
        }
        return buffer;
    }
    
    pub fn gen_ppr(& self, pk: usize) -> (Vec<u8>, Vec<u8>, B_OFFSET, Duration)
    {
        let mut par0 : Vec<u8> = Vec::new();
        let mut par1 : Vec<u8> = Vec::new();
        
        let mut zeta = Z_OFFSET;
        let mut pset = vec![0u8; IV_SIZE];
        let pos = pk * ISIZE;
        
        self.os_random(& mut pset);
        zeta.copy_from_slice(& pset[pos + ISQRT .. pos + ISIZE]);

        let start = Instant::now();
        
        for (i, indice) in pset.chunks_exact_mut(ISIZE).enumerate()
        {
            let rbit = indice[0];
            indice[0 .. ISQRT].copy_from_slice(& self.prep_block[i]);
            
            if i == pk 
            {
                par1.extend_from_slice(indice);
            }
            else if 1 == 1 & rbit
            {
                par0.extend_from_slice(indice);
                par1.extend_from_slice(indice);
            }
        }

        let finis = Instant::now();
        
        return (par0, par1, zeta, finis - start)
    }

    pub fn gen_pir(& self, pos: usize) -> (Vec<u8>, Vec<u8>, Duration)
    {
        let mut base = vec![0u8; 2 * HSIZE / 8];
        self.os_random(& mut base);

        let start = Instant::now();

        let str_a = BitVec::<_, Msb0>::from_vec(base.clone());
        let mut str_b = BitVec::<_, Msb0>::from_vec(base);
        str_b.set(pos, !str_a[pos]);

        let finis = Instant::now();

        return (str_a.into_vec(), str_b.into_vec(), finis - start);
    }
}
    
    
pub struct Storage {
    map_space: Mmap,
    arr_block: File,
    xor_regis: [SIMD; BUNIT],
    prs_block: [Block; SSIZE],
    par_block: [INDX; SSIZE],
}

impl Storage {

    pub fn new() -> Self
    {
        let arr_block = File::open("data").expect("open data fail");
        let map_space = unsafe { Mmap::map(& arr_block).expect("map fail") };
        let xor_regis = [LANE::splat(0); BUNIT];

        let block = GenericArray::from([0u8; 16]);
        let mut prs_block : [Block; SSIZE] = [block; SSIZE];
        let mut par_block : [INDX; SSIZE] = [0; SSIZE];

        for i in 0 .. SSIZE
        {
            let val = & (i as SQRT).to_be_bytes();
            prs_block[i][.. ISQRT].copy_from_slice(val);
            par_block[i] = (i as INDX) << LSIZE;
        }

        Self {map_space, arr_block, xor_regis, prs_block, par_block}
    }

    pub fn select(& mut self, i: usize) -> Duration
    {
        let mut block = vec![0u8; BSIZE];

        block.copy_from_slice(& self.map_space[i * BSIZE .. (i + 1) * BSIZE]);

        let start = Instant::now();

        for (i, chunk) in block.chunks(USIZE).enumerate()
        {
            self.xor_regis[i] ^= LANE::from_slice_unaligned(chunk);
        }

        let finis = Instant::now();
        
        return finis - start;
    }

    pub fn result(& mut self) -> Vec<u8>
    {
        let mut block = vec![0u8; BSIZE];

        for (i, chunk) in block.chunks_exact_mut(USIZE).enumerate()
        {
            self.xor_regis[i].write_to_slice_unaligned(chunk);
            self.xor_regis[i] = LANE::splat(0);
        }

        return block;
    }

    pub fn prep(& mut self, key: & [u8]) -> Vec<u8>
    {
        let cipher = Aes128::new(key.into());

        let mut blocks = self.prs_block;
        cipher.encrypt_blocks(& mut blocks);

        for i in 0 .. SSIZE
        {
            let index = parse_index_2(self.par_block[i], & blocks[i][.. ISQRT]);
            self.select(index);
        }

        return self.result();
    }

    pub fn parity(& mut self, arr: &[u8], flag: bool) -> (Vec<u8>, Duration)
    {        
        let parse = if flag {parse_index_1} else {parse_index_2};
        let steps = if flag {ISIZE} else {ISQRT};

        let mut t_comp = Duration::from_secs(0);
        
        for (i, chunk) in arr.chunks(steps).enumerate()
        {
            t_comp += self.select(parse(self.par_block[i], chunk));
        }

        return (self.result(), t_comp)
    }
}





extern "C" {
    fn add_byte_arrays(arr1: *mut u8, arr2: *const u8, size: usize);
}


pub struct StoragePlus {
    map_space: Mmap,
    arr_block: File,
    mod_regis: [PINT; BSIZE / MSIZE],
    prs_block: [Block; SSIZE],
    par_block: [INDX; SSIZE],
}

impl StoragePlus {

    pub fn new() -> Self
    {
        let arr_block = File::open("data").expect("open data fail");
        let map_space = unsafe { Mmap::map(& arr_block).expect("map fail") };
        let mod_regis = [0; BSIZE / MSIZE];

        let block = GenericArray::from([0u8; 16]);
        let mut prs_block : [Block; SSIZE] = [block; SSIZE];
        let mut par_block : [INDX; SSIZE] = [0; SSIZE];

        for i in 0 .. SSIZE
        {
            let val = & (i as SQRT).to_be_bytes();
            prs_block[i][.. ISQRT].copy_from_slice(val);
            par_block[i] = (i as INDX) << LSIZE;
        }

        Self {map_space, arr_block, mod_regis, prs_block, par_block}
    }

    pub fn select(& mut self, i: usize) -> Duration
    {
        let mut block = vec![0u8; BSIZE];

        block.copy_from_slice(& self.map_space[i * BSIZE .. (i + 1) * BSIZE]);

        let start = Instant::now();

        for (i, chunk) in block.chunks_exact(MSIZE).enumerate()
        {
            let chunk : [u8; MSIZE] = chunk.try_into().unwrap();
            self.mod_regis[i] += PINT::from_be_bytes(chunk);
        }

        let finis = Instant::now();

        return finis - start;
    }

    pub fn result(& mut self) -> Vec<u8>
    {
        let mut block = vec![0u8; BSIZE];

        for (i, chunk) in block.chunks_exact_mut(MSIZE).enumerate()
        {
            chunk.copy_from_slice(& self.mod_regis[i].to_be_bytes());
            self.mod_regis[i] = 0;
        }

        return block;
    }

    pub fn prep(& mut self, key: & [u8]) -> Vec<u8>
    {
        let cipher = Aes128::new(key.into());

        let mut blocks = self.prs_block;
        cipher.encrypt_blocks(& mut blocks);

        // println!("list index:");

        for i in 0 .. SSIZE
        {
            let index = parse_index_2(self.par_block[i], & blocks[i][.. ISQRT]);
            self.select(index);

            // println!("{:?}", index);
        }

        return self.result();
    }

    pub fn parity(& mut self, arr: &[u8], flag: bool) -> (Vec<u8>, Duration)
    {        
        let parse = if flag {parse_index_1} else {parse_index_2};
        let steps = if flag {ISIZE} else {ISQRT};

        let mut t_comp = Duration::from_secs(0);
        
        for (i, chunk) in arr.chunks(steps).enumerate()
        {
            t_comp += self.select(parse(self.par_block[i], chunk));
        }

        return (self.result(), t_comp);
    }
}




extern "C" {
    fn xor_byte_arrays(arr1: *mut u8, arr2: *const u8, size: usize);
}



pub struct HintStorage {
    map_space: MmapMut,
    arr_block: File,
    pub xor_regis: Vec<u8>
}

impl HintStorage {

    pub fn new() -> Self
    {
        let arr_block = OpenOptions::new()
            .read(true)
            .write(true)
            .open("ehint").expect("ehint kset fail");

        let map_space = unsafe { MmapMut::map_mut(& arr_block).expect("map fail") };

        let xor_regis = vec![0u8; ESIZE];

        Self {map_space, arr_block, xor_regis}
    }

    pub fn select(& mut self, i: usize) -> Duration
    {
        let mut block = vec![0u8; ESIZE];

        block.copy_from_slice(& self.map_space[i * ESIZE .. (i + 1) * ESIZE]);

        let size = self.xor_regis.len();

        let start = Instant::now();

        unsafe {
            xor_byte_arrays(self.xor_regis.as_mut_ptr(), block.as_ptr(), size)
        }

        let finis = Instant::now();

        return finis - start;
    }

    pub fn parity(& mut self, arr: Vec<u8>) -> (Duration, Vec<u8>)
    {        
        let bitstr = BitVec::<_, Msb0>::from_vec(arr.to_vec());

        let mut t_comp = Duration::from_secs(0);
        
        for i in 0 .. bitstr.len()
        {
            if bitstr[i]
            {
                t_comp += self.select(i);
            }
        }

        return (t_comp, self.xor_regis.clone());
    }

    pub fn clear_regis(& mut self)
    {
        self.xor_regis = vec![0u8; ESIZE];
    }

    pub fn write(& mut self, pos: usize, arr: &[u8])
    {
        let block = & mut self.map_space[pos * ESIZE .. (pos + 1) * ESIZE];

        block.copy_from_slice(& arr);
    }
}