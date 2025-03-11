
#![allow(arithmetic_overflow)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use rand::rngs::OsRng;
use rand::RngCore;
use std::{collections::HashMap, time::Duration};
use std::time::Instant;
use std::thread;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Scalar, All};
use aes::Aes128;
use aes::cipher::BlockEncrypt;
use aes::cipher::KeyInit;
use aes::cipher::generic_array::GenericArray;


pub type SINT = u8;
pub type PINT = u16;
pub const M_SIZE : usize = 2;
pub const B_SQRT : PINT = 64000 + SINT::MAX as PINT;
pub const F_EXPA : usize = 33;

pub struct AHE
{
    secp: Secp256k1<All>,
    sv: Scalar,
    sk: SecretKey,
    pk: PublicKey,
    table: HashMap<[u8; 33], PINT>,
    ms: PublicKey,
    aes: Aes128,
}


pub const BASE : [u8; 33] = [2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160, 98, 149, 206, 135, 11, 7, 2, 155, 252, 219, 45, 206, 40, 217, 89, 242, 129, 91, 22, 248, 23, 152];



impl Clone for AHE {
    fn clone(&self) -> Self {
        AHE {
            secp: self.secp.clone(),
            sv: self.sv.clone(),
            sk: self.sk.clone(),
            table: self.table.clone(),
            pk: self.pk.clone(),
            ms: self.ms.clone(),
            aes: self.aes.clone(),
        }
    }
}


fn pad_array(input: &[u8]) -> [u8; 32]
{
    let zero_cnt = 32 - input.len();

    let mut zero_pad = vec![0u8; zero_cnt];

    zero_pad.extend(input);

    let mut result = [0u8; 32];

    result.copy_from_slice(& zero_pad);

    return result;
}



impl AHE
{
    pub fn new(secret: [u8; 32]) -> Self
    {
        let secp = Secp256k1::new();

        let sv = Scalar::from_be_bytes(secret).unwrap();
        let sk = SecretKey::from_slice(& secret).unwrap();
        let pk = PublicKey::from_secret_key(& secp, & sk);

        let mut table: HashMap<[u8; 33], PINT> = HashMap::new();

        let G = PublicKey::from_slice(& BASE).unwrap();
        let mut step = G.clone();

        for i in 1 .. B_SQRT
        {
            table.insert(step.serialize(), i);
            step = step.combine(& G).unwrap();
        }

        let raw = pad_array(& B_SQRT.to_be_bytes());
        let ms = PublicKey::from_secret_key(& secp, & SecretKey::from_slice(& raw).unwrap()).negate(& secp);

        let key = GenericArray::from_slice(& [117u8; 16]);
        let aes = Aes128::new(& key);

        Self { secp, sv, sk, pk, table, ms, aes }
    }


    pub fn prand_val(& self, bid: usize, gid: usize) -> [u8; 32]
    {
        let head = GenericArray::from((bid as u128).to_be_bytes());
        let tail = GenericArray::from((gid as u128).to_be_bytes());

        let mut buffer = [head, tail];
        let mut result = [0u8; 32];
        self.aes.encrypt_blocks(& mut buffer);

        result[.. 16].copy_from_slice(& buffer[0][.. 16]);
        result[16 ..].copy_from_slice(& buffer[1][.. 16]);

        return result
    }


    pub fn encrypt(& self, r_val: [u8; 32], chunk: &[u8]) -> [u8; 33]
    {
        let raw = pad_array(chunk);
        let _m = Scalar::from_be_bytes(raw).expect("text encoding");
        
        let r_raw = Scalar::from_be_bytes(r_val).unwrap();
        let p_twk = self.pk.mul_tweak(& self.secp, & r_raw).expect("random tweak");
        let c2 = p_twk.add_exp_tweak(& self.secp, & _m).expect("");

        return c2.serialize();
    }


    pub fn decrypt(& self, r_val: [u8; 32], enc: &[u8]) -> PINT
    {
        let r_key = SecretKey::from_slice(& r_val).unwrap();
        let c1 = PublicKey::from_secret_key(& self.secp, & r_key);
        let c2 = PublicKey::from_slice(& enc).unwrap();

        let n1 = c1.negate(& self.secp);
        let t1 = n1.mul_tweak(& self.secp, & self.sv).unwrap();
        let mut mp = c2.combine(& t1).unwrap();


        for i in 0 .. B_SQRT
        {
            let key = mp.serialize();
            
            if self.table.contains_key(& key)
            {
                return i * B_SQRT + self.table.get(& key).unwrap();
            }
            mp = mp.combine(& self.ms).unwrap();
        }

        println!("dead end");

        return 0
    }

    pub fn addition(& self, _point_a: [u8; 64], _point_b: [u8; 64]) -> [u8; 64]
    {
        return [0u8; 64];
    }
}



fn thread_decrypt(_it: usize, ahe: AHE, bid: usize, idx: &[usize], enc: &[u8]) -> (Vec<u8>, Duration)
{
    if idx.len() != enc.len() / F_EXPA
    {
        println!("DEAD DEAD DEAD");
    }

    let mut _res = vec![0u8; 0];

    let mut t_comp = Duration::from_secs(0);
    
    
    for (i, chunk) in enc.chunks(F_EXPA).enumerate()
    {
        let r_val = ahe.prand_val(bid, idx[i]);
        let start = Instant::now();
        let m_chunk = ahe.decrypt(r_val, chunk);
        t_comp += Instant::now() - start;

        _res.extend(m_chunk.to_be_bytes());
    }

    return (_res, t_comp);
}


fn thread_encrypt(_it: usize, ahe: AHE, bid: usize, idx: &[usize], pln: &[u8]) -> (Vec<u8>, Duration)
{
    if idx.len() != pln.len() / M_SIZE
    {
        println!("DEAD DEAD DEAD");
    }

    let mut _enc = vec![0u8; 0];
    
    let mut t_comp = Duration::from_secs(0);

    for (i, chunk) in pln.chunks(M_SIZE).enumerate()
    {
        let r_val = ahe.prand_val(bid, idx[i]);
        let start = Instant::now();
        let e_chunk = ahe.encrypt(r_val, chunk);
        t_comp += Instant::now() - start;

        _enc.extend(e_chunk);
    }

    return (_enc, t_comp);
}


pub fn parallel_decrypt(ahe: &AHE, bid: usize, enc: &[u8]) -> (Vec<u8>, Duration)
{
    let n_thread : usize = 8;
    
    let esize = enc.len() / n_thread;

    let gsize = esize / F_EXPA;

    let mut handles = vec![];

    for i in 0 .. n_thread
    {
        let mut etest = vec![0u8; esize];

        let gtest : Vec<usize> = (i * gsize .. (i + 1) * gsize).collect();

        etest.copy_from_slice(& enc[i * esize .. (i + 1) * esize]);

        let ahe_clone = ahe.clone();
    
        let handle = thread::spawn(move || {
            
            return thread_decrypt(i, ahe_clone, bid, & gtest, & etest);
        });
        
        handles.push((i, handle));
    }

    let mut res = vec![0u8; 0];

    let mut _time = Duration::from_secs(0);


    for (_i, handle) in handles
    {
        let (t_res, t_comp) = handle.join().unwrap();

        _time += t_comp;

        res.extend(t_res);
    }

    return (res, _time / (n_thread as u32));
}



pub fn parallel_encrypt(ahe: &AHE, bid: usize, text: &[u8]) -> (Vec<u8>, Duration)
{
    let n_thread : usize = 8;
    
    let esize = text.len() / n_thread;

    let gsize = esize / M_SIZE;

    let mut handles = vec![];

    for i in 0 .. n_thread
    {
        let mut etest = vec![0u8; esize];

        let gtest : Vec<usize> = (i * gsize .. (i + 1) * gsize).collect();

        etest.copy_from_slice(& text[i * esize .. (i + 1) * esize]);

        let ahe_clone = ahe.clone();
    
        let handle = thread::spawn(move || {
            
            return thread_encrypt(i, ahe_clone, bid, & gtest, & etest);
        });
        
        handles.push((i, handle));
    }

    let mut res = vec![0u8; 0];

    let mut _time = Duration::from_secs(0);

    for (_i, handle) in handles
    {
        let (t_res, t_comp) = handle.join().unwrap();

        _time += t_comp;

        res.extend(t_res);
    }

    return (res, _time / (n_thread as u32));
}




pub fn estimate_update(ahe: &AHE)
{
    let mut rando = [0u8; 32];

    OsRng.fill_bytes(&mut rando[16 ..]);

    let raw = Scalar::from_be_bytes(rando).unwrap();

    
    let mut handles = vec![];
    
    for i in 0 .. 8
    {
        let ahe_clone = ahe.clone();
        
        let handle = thread::spawn(move || {
            
            let start = Instant::now();
            for _ in 0 .. 10
            {
                ahe_clone.pk.mul_tweak(& ahe_clone.secp, & raw).expect("epsilon tweak");
                ahe_clone.pk.combine(& ahe_clone.pk).unwrap();
            }
            let finis = Instant::now();

            return finis - start;
        });
    
        handles.push((i, handle));
    }

    for (_i, handle) in handles
    {
        let _time = handle.join().unwrap();

        println!("{:?}", _time);
    }
}




fn main()
{
    let mut secret = [0u8; 32];

    OsRng.fill_bytes(&mut secret);
    
    let ahe = AHE::new(secret);

    estimate_update(& ahe);
}