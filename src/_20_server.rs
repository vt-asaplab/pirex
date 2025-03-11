
#![allow(dead_code)]

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::time::Instant;
use std::time::Duration;
use std::fs::OpenOptions;

mod libs;
use libs::*;

const LAMBDA: usize = 128;


// CK20


fn simulate_server_computation() -> (Duration, Vec<u8>)
{
    let crypto = Crypto::new();
    let mut store = Storage::new();

    let mut key = Z_KEYSET;
    let mut set = vec![0u8; IV_SIZE];

    crypto.os_random(& mut set);

    for chunk in set.chunks_exact_mut(ISQRT)
    {
        chunk[0] &= FRONT;
    }

    let start = Instant::now();

    for i in 1 .. SSIZE
    {
        let (res, _t) = crypto.key_val(& key, i % SSIZE);
        key[i % 16] = res[0]
    }

    store.parity(& set, true);

    let result = store.result();

    let finis = Instant::now();

    return (finis - start, result);
}

fn handle_client(mut stream: TcpStream) -> (Duration, Instant)
{    
    let block = vec![0u8; BSIZE * LAMBDA];
    
    let (time, _) = simulate_server_computation();

    let finis = Instant::now();

    stream.write_all(& block).expect("response fail");

    return (time * (LAMBDA as u32), finis);
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
    let mut key = vec![0u8; LAMBDA * (SSIZE - 1) * ISIZE];

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
                    let comp = format!("CK20 server comp elapse: {:?} \n", t_comp / n_test);
                    let dead = format!("CK20 server dead elapse: {:?} \n", t_dead / n_test);

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
