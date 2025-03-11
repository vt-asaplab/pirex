
#![allow(dead_code)]
#![allow(overflowing_literals)]

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::time::Instant;
use std::fs::OpenOptions;
use aes::Aes128;
use aes::cipher::BlockEncrypt;
use aes::cipher::KeyInit;
use aes::cipher::generic_array::GenericArray;

mod libs;
use libs::*;

const LAMBDA: usize = 128;


// CK20


fn simulate_client_search_key() -> Duration // worst case: hint sizes
{
    let key = GenericArray::from([0u8; 16]);
    
    let mut block = GenericArray::from([42u8; 16]);

    let cipher = Aes128::new(&key);

    let index = (12482 % NSIZE) as INDX;

    let crypto = Crypto::new();

    let mut key = Z_KEYSET;

    crypto.os_random(& mut key);

    let start = Instant::now();
    
    let (pk, offset) = crypto.ppr_val(index);

    let (res, _t_val) = crypto.key_val(& key, pk);

    while offset != res
    {
        crypto.os_random(& mut key);

        for _ in 0 .. LSIZE // poly log membership
        {
            cipher.encrypt_block(&mut block);
        }
    }

    let finis = Instant::now();

    return finis - start;
}

fn simulate_client_create_key() -> Duration
{
    let crypto = Crypto::new();
    
    let mut key = Z_KEYSET;

    let start = Instant::now();
    
    crypto.os_random(& mut key);

    let (res, _t) = crypto.key_val(& key, 0);

    key[1] = res[0]; // no loop since any index can be shifted

    if 0 == key[1]
    {
        key[0] = 3;
    }

    let finis = Instant::now();

    return finis - start
}


fn simulate_client_create_query() -> Duration // generate all
{
    let mut key = Z_KEYSET;
    
    let crypto = Crypto::new();

    let start = Instant::now();
    
    for i in 0 .. (SSIZE - 1)
    {
        let (res, _t) = crypto.key_val(& key, i);
        key[1]= res[0];
    }

    let finis = Instant::now();

    return finis - start;
}


fn simulate_client_communication() -> Duration
{
    let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("stream fail");

    let pset = vec![0u8; LAMBDA * (SSIZE - 1) * ISIZE];
    let mut block = vec![0u8; BSIZE * LAMBDA];

    let start = Instant::now();

    stream.write_all(& pset).expect("request fail");
    stream.read_exact(& mut block).expect("response fail");

    let finis = Instant::now();

    return finis - start;
}


fn simulate_client_recover() -> Duration
{
    let crypto = Crypto::new();

    let mut block = vec![0u8; BSIZE];
    let mut regis = [LANE::splat(0); BUNIT];
    
    crypto.os_random(& mut block);

    let start = Instant::now();

    for _ in 0 .. 2
    {
        for (j, chunk) in block.chunks(USIZE).enumerate()
        {
            regis[j] ^= LANE::from_slice_unaligned(chunk);
        }
    }
        
    for (i, chunk) in block.chunks_exact_mut(USIZE).enumerate()
    {
        regis[i].write_to_slice_unaligned(chunk);
    }

    let finis = Instant::now();

    return finis - start;
}

fn main()
{
    let mut t_comp = Duration::from_secs(0);
    let mut t_band = Duration::from_secs(0);

    let n_test = 1;
    let n_time = LAMBDA as u32;

    for _ in 0 .. n_test
    {   
        t_comp += n_time * simulate_client_search_key();

        t_comp += n_time * simulate_client_create_key();
        
        for _ in 0 .. 2
        {
            t_band += simulate_client_communication(); // bandwidth

            t_comp += n_time * simulate_client_create_query();
            
            t_comp += n_time * simulate_client_recover();
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("client_res_auto").unwrap();

    let comp = format!("CK20 client comp elapse: {:?} \n", t_comp / n_test);
    let band = format!("CK20 client band elapse: {:?} \n", t_band / n_test);

    file.write_all(comp.as_bytes()).unwrap();
    file.write_all(band.as_bytes()).unwrap();
}