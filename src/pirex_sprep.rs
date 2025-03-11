
#![allow(dead_code)]

use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::time::Duration;
use std::time::Instant;
use std::thread;
use rand::Rng;

mod libs;
use libs::*;


fn simulate_keygen() -> Duration
{
    let mut key = Z_KEYSET;

    let crypto = Crypto::new();

    let start = Instant::now();

    for i in 0 .. 2 * LSIZE * HSIZE // dummy path from root PRF
    {
        let (res, _t) = crypto.key_val(& key, i % SSIZE);
        key[1] = res[0];
    }

    let finis = Instant::now();

    return finis - start;
}


fn simulate_select() -> Duration
{
    let mut array = Vec::new();
    let mut urand = rand::thread_rng();

    for i in 0 .. SSIZE * HSIZE 
    {
        let v: usize = urand.gen_range(0 .. NSIZE);
        array.push((i % SSIZE) * SSIZE + v);
    }

    let mut store = Storage::new();

    let start = Instant::now();

    for each in array
    {
        store.select(each);
    }

    let finis = Instant::now();

    return finis - start;
}


fn process_thread(it: usize, kset: Vec<u8>) -> (Duration, Vec<u8>)
{
    let mut storage = Storage::new();

    let mut hint = vec![0u8; BSIZE * REGION];

    println!("thread {it} starts");

    // let lp = simulate_keygen();
    // let ck = simulate_select();

    let start = Instant::now();

    for i in 0 .. REGION
    {
        let key = & kset[i * KSIZE .. (i + 1) * KSIZE];
        let pos = & mut hint[i * BSIZE .. (i + 1) * BSIZE];
        let val = & storage.prep(& key);
        
        pos.copy_from_slice(val);
    }

    let finis = Instant::now();

    println!("thread {it}: {:?}", finis - start);

    return (finis - start, hint);
}

fn handle_client(mut stream: TcpStream)
{    
    let mut handles = vec![];
    
    for i in 0 .. THREAD
    {
        let mut kset = vec![0u8; KSIZE * REGION];
        
        stream.read_exact(& mut kset).expect("request fail");
        
        let handle = thread::spawn(move || {
            return process_thread(i, kset);
        });
        
        handles.push((i, handle));
    }

    let mut result: [Vec<u8>; THREAD] = Default::default();
    let mut total = [Duration::from_secs(0); 3];

    for (i, handle) in handles
    {
        let (xx, res) = handle.join().unwrap();
        result[i] = res;
        total[0] += xx;
        // total[1] += ck;
        // total[2] += lp;
    }

    for i in 0 .. THREAD
    {
        stream.write_all(& result[i]).expect("response fail");
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("server_prep_auto").unwrap();

    let xx = format!("server XX prep elapse: {:?} \n", total[0] / (THREAD as u32));
    // let ck = format!("server CK prep elapse: {:?} \n", total[1] / (THREAD as u32));
    // let lp = format!("server LP prep elapse: {:?} \n", total[2] / (THREAD as u32));

    file.write_all(xx.as_bytes()).unwrap();
    // file.write_all(ck.as_bytes()).unwrap();
    // file.write_all(lp.as_bytes()).unwrap();
}

fn main()
{
    let listener = TcpListener::bind(SERVER_ADDRESS).expect("error binding");

    loop {
        match listener.accept() {
            
            Ok((stream, _)) => handle_client(stream),
            
            Err(e) => println!("error connection: {e}")
        }
    }
}
