
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::ops::DerefMut;
use memmap::MmapMut;
use std::time::Instant;

mod libs;
use libs::*;

fn main()
{
    let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("stream fail");
    let crypto = Crypto::new();

    let mut fs_key = File::create("kset").expect("init kset fail");
    let mut fs_par = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("hint").expect("init hint fail");

    fs_par.set_len((BSIZE * HSIZE) as u64).expect("error hint size");

    let mut kset = vec![0u8; KSIZE * HSIZE];
    let mut disk = unsafe { MmapMut::map_mut(& fs_par).expect("map fail") };
    let hint = disk.deref_mut();

    let start = Instant::now();

    crypto.os_random(& mut kset);
    stream.write_all(& kset).expect("request fail");
    stream.read_exact(hint).expect("response fail"); 
    
    let finis = Instant::now();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("client_prep_auto").unwrap();

    file.write_all(format!("client prep elapse {:?} \n", finis - start).as_bytes()).unwrap();
    
    fs_key.write_all(& kset).expect("write keys fail");
    fs_par.flush().expect("flush fail");
}
