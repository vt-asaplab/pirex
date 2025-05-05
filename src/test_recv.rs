

use std::io::Read;
use std::io::Write;
use std::net::TcpListener;

fn main()
{
    let address = "127.0.0.1:8111";

    let listener = TcpListener::bind(address).expect("error binding");

    loop {
        match listener.accept() {
            
            Ok((mut stream, _)) => 
            {
                let mut buffer = vec![0u8; 22528];
                stream.read_exact(& mut buffer).unwrap();
                stream.write_all(& buffer).unwrap();
            },

            Err(e) => println!("error connection: {}", e)
        }
    }
}