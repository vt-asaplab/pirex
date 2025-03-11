
#![allow(dead_code)]

use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::ops::DerefMut;
use memmap::MmapMut;

mod libs;
use libs::*;


fn process_thread(it: usize, kset: Vec<u8>) -> (Duration, Vec<u8>)
{
    let mut storage = StoragePlus::new();

    let mut hint = vec![0u8; BSIZE * REGION];

    println!("thread {it} starts");

    let start = Instant::now();

    for i in 0 .. REGION
    {
        let key = & kset[i * KSIZE .. (i + 1) * KSIZE];
        let pos = & mut hint[i * BSIZE .. (i + 1) * BSIZE];
        let val = & storage.prep(& key);

        // println!("hint: {:?}", val);
        
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
    }

    for i in 0 .. THREAD
    {
        stream.write_all(& result[i]).expect("response fail");
    }

    // ----- START RECEIVING ENCRYPTED PARITY -----

    let mut pfile = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("ehint").expect("init hint fail");

    let len_buffer = HSIZE * ESIZE * 2;

    pfile.set_len(len_buffer as u64).expect("error hint size");

    let mut mount = unsafe { MmapMut::map_mut(& pfile).expect("map fail") };
    let ehint = mount.deref_mut();

    stream.read_exact(& mut ehint[.. (len_buffer / 2)]).expect("response fail"); 

    ehint[(len_buffer / 2) ..].fill_with(|| 0);

    pfile.flush().expect("flush fail");

    // ----- FINISH RECEIVING ENCRYPTED PARITY -----

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("server_prep_auto").unwrap();

    let xx = format!("server XX prep elapse: {:?} \n", total[0] / (THREAD as u32));

    file.write_all(xx.as_bytes()).unwrap();
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
