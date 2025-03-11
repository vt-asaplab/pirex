
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Instant;
use std::time::Duration;


mod libs;
use libs::*;


fn handle_client(storage: & mut Storage, mut stream: TcpStream)
{
    let mut header = [0u8; 4];
    let mut buffer = vec![0u8; 8 * IV_SIZE];
    let mut result = vec![0u8; 0];
    let mut acknown = [0u8; 1];

    println!("\n===== new request =====\n");

    let mut t_comp = Duration::from_secs(0);

    // full stream

    stream.read_exact(& mut header).expect("request fail");
    let full_length = u32::from_be_bytes(header) as usize;
    stream.read_exact(& mut buffer[.. full_length]).unwrap();
    stream.write(& acknown).expect("acknown fail");

    // stream ends

    let mut index = 0;

    for i in 0 .. 4
    {
        let size_bytes: [u8; 4] = [
            buffer[index],
            buffer[index + 1],
            buffer[index + 2],
            buffer[index + 3],
        ];

        let length = u32::from_be_bytes(size_bytes) as usize;
        // println!("recv {length}");

        let idx_range = & buffer[index + 4 .. index + 4 + length];
        let (parity, _t) = storage.parity(& idx_range, i & 1 == 1);
        result.extend_from_slice(& parity);
        
        t_comp += _t;
        index += 4 + length;
    }

    let start = Instant::now();
    stream.write_all(& result).expect("response fail");
    stream.read_exact(& mut acknown).expect("acknown fail");
    let finis = Instant::now();

    println!("inbound delay: {:?}", finis - start);
    println!("server comp delay: {:?}", t_comp / 2);
}

fn main()
{
    let listener = TcpListener::bind(SERVER_ADDRESS).expect("error binding");
    
    let mut storage = Storage::new();

    println!("Test DB: 2^{:?} entries {:?} KB", LSIZE * 2, BSIZE / 1024);

    loop {
        match listener.accept() {
            
            Ok((stream, _)) => 
            {
                handle_client(& mut storage, stream);
            },

            Err(e) => println!("error connection: {}", e)
        }
    }
}
