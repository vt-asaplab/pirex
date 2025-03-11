
#![allow(dead_code)]

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::time::Duration;
use std::time::Instant;
use std::fs::OpenOptions;
use rand::Rng;

mod libs;
use libs::*;


// LP22


fn simulate_server_computation() -> (Duration, Vec<u8>)
{
    let crypto = Crypto::new();
    let mut store = Storage::new();

    let mut key = Z_KEYSET;
    let result = vec![0u8; BSIZE];
    
    // let mut vector = Vec::new();
    let mut array = Vec::new();
    let mut urand = rand::thread_rng();

    for i in 0 .. LSIZE * LSIZE * SSIZE 
    {
        let v: usize = urand.gen_range(0 .. SSIZE);
        array.push((i % SSIZE) * SSIZE + v);
    }

    let mut c = 0;
    let mut p = 4;
    let mut k = 1;
    let start = Instant::now();

    for i in 2 .. (1 + LSIZE)
    {
        for j in 0 .. (p * k) as usize
        {
            let (res, _t) = crypto.key_val(& key, 1);
            key[i * j % 16] = res[0];
            store.select(array[c]);
            c = c + 1;
        }
        p = p * 2;
        k = k + 1;
    }

    let finis = Instant::now();

    return (finis - start, result);
}


fn handle_client(mut stream: TcpStream) -> (Duration, Instant)
{    
    let block = vec![117u8; BSIZE * SSIZE];

    let (total, _) = simulate_server_computation();

    println!("LP finish computation");

    let finis = Instant::now();

    stream.write_all(& block).expect("response fail");

    return (total, finis);
}


fn main()
{
    let listener = TcpListener::bind(SERVER_ADDRESS).expect("error binding");

    let mut t_comp = Duration::from_secs(0);
    let mut t_dead = Duration::from_secs(0);
    let mut t_step = 0;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("server_res_auto").unwrap();

    let n_test = 1;
    let mut key = vec![0u8; KSIZE * (LSIZE - 1)];

    loop {
        match listener.accept() {
            
            Ok((mut stream, _)) => 
            {
                stream.read_exact(& mut key).expect("request fail");
                
                let start = Instant::now();
                
                let (ptime, finis) = handle_client(stream);
                
                t_comp += ptime;
                t_dead += finis - start;
                t_step += 1;

                if t_step % (2 * n_test) == 0
                {
                    let comp = format!("LP22 server comp elapse: {:?} \n", t_comp / n_test);
                    let dead = format!("LP22 server dead elapse: {:?} \n", t_dead / n_test);

                    file.write_all(comp.as_bytes()).unwrap();
                    file.write_all(dead.as_bytes()).unwrap();
                    file.flush().unwrap();
                    
                    t_comp = Duration::from_secs(0);
                    t_dead = Duration::from_secs(0);
                }
            },

            Err(e) => println!("error connection: {}", e)
        }
    }
}
