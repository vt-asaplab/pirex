
#![allow(dead_code)]
#![allow(overflowing_literals)]

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;
use std::time::Duration;
use std::fs::OpenOptions;
use aes::Aes128;
use aes::cipher::BlockEncrypt;
use aes::cipher::KeyInit;
use aes::cipher::generic_array::GenericArray;

mod libs;
use libs::*;


// LP22


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
        
        for _ in 0 .. LSIZE // dummy path from root PRF
        {
            cipher.encrypt_block(&mut block);
        }
    }

    let finis = Instant::now();

    return finis - start;
}


fn simulate_client_create_key() -> Duration
{
    let key = GenericArray::from([0u8; 16]);
    
    let mut block = GenericArray::from([42u8; 16]);

    let cipher = Aes128::new(&key);

    let start = Instant::now();

    for _ in 0 .. LSIZE // need the path to get shift value
    {
        cipher.encrypt_block(&mut block);
    }

    let finis = Instant::now();

    return finis - start;
}


fn simulate_client_create_query() -> Duration // pick sibling nodes on path
{   
    let key = GenericArray::from([0u8; 16]);
    
    let mut block = GenericArray::from([42u8; 16]);

    let cipher = Aes128::new(&key);

    let start = Instant::now();
    
    for _ in 0 .. LSIZE - 1 
    {
        cipher.encrypt_block(&mut block);
    }

    let finis = Instant::now();

    return finis - start;
}


fn simulate_client_communication() -> Duration
{
    let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("stream fail");

    let key = vec![117u8; KSIZE * (LSIZE - 1)];
    let mut block = vec![0u8; BSIZE * SSIZE];

    let start = Instant::now();

    stream.write_all(& key).expect("request fail");
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

    for _ in 0 .. n_test
    {   
        t_comp += simulate_client_search_key();
        
        t_comp += simulate_client_create_key();
        
        for _ in 0 .. 2
        {                
            // t_comp += simulate_client_create_query();
            
            // println!("LP start sending");
            
            t_band += simulate_client_communication(); // bandwidth
            
            t_comp += simulate_client_recover();
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("client_res_auto").unwrap();

    let comp = format!("LP22 client comp elapse: {:?} \n", t_comp / n_test);
    let band = format!("LP22 client band elapse: {:?} \n", t_band / n_test);

    file.write_all(comp.as_bytes()).unwrap();
    file.write_all(band.as_bytes()).unwrap();
}
