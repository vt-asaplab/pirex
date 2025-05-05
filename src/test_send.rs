
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;
use std::time::Duration;

fn main()
{
    let address = "127.0.0.1:8111";
    let mut stream = TcpStream::connect(address).expect("stream fail");


    let mut total = Duration::from_secs(0);

    for i in 0 .. 1
    {
        let inp = vec![77u8; 22528];
        let mut ack = [0u8; 1];

        println!("run test {}", i);
        let start = Instant::now();
        stream.write_all(& inp).unwrap();
        stream.read_exact(& mut ack).unwrap();
        let finis = Instant::now();

        total += finis - start;
        
        println!("network delay: {:?}", total / (i + 1));
    }
}