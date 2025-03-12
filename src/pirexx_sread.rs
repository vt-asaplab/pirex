
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Duration;
use std::time::Instant;

mod libs;
use libs::*;


fn handle_client(storage: & mut StoragePlus, hbuffer: & mut HintStorage, mut stream: TcpStream)
{
    // println!("\n===== new request =====\n");

    let mut t_sum = Duration::from_secs(0);
    let mut header = [0u8; 4];
    let mut buffer = vec![0u8; IV_SIZE * 2];
    let mut acknown = [0u8; 1];

    // first xor

    let mut base_01 = vec![0u8; PSIZE];
    stream.read_exact(& mut base_01).expect("request pir fail");
    stream.write_all(& acknown).unwrap();
    let (xor_t1, res_01) = hbuffer.parity(base_01);
    
    // second xor

    let mut base_02 = vec![0u8; PSIZE];
    stream.read_exact(& mut base_02).expect("request pir fail");
    stream.write_all(& acknown).unwrap();
    let (xor_t2, res_02) = hbuffer.parity(base_02);

    println!("requests xorpir nbytes {:?}", PSIZE * 2);
    println!("_requests should takes {:?} ms", PSIZE * 2 / 5000);
    
    println!("response xorpir nbytes {:?}", res_02.len() + res_01.len());
    println!("_response should takes {:?} ms", (res_02.len() + res_01.len()) / 5000);
    
    // normal query

    let mut total_length = 0;
    let mut normal_resp = 0;

    for i in 0 .. 4
    {
        stream.read_exact(& mut header).expect("request fail");
        let length = u32::from_be_bytes(header) as usize;
        stream.read_exact(& mut buffer[.. length]).unwrap();

        // println!("recv nbytes {length}");

        total_length += length;

        let (parity, _t) = storage.parity(& buffer[.. length], i & 1 == 1);
        stream.write_all(& parity).unwrap();

        normal_resp += parity.len();

        t_sum += _t;
    }

    println!("requests normal nbytes {:?}", total_length);
    println!("_requests should takes {:?} ms", total_length / 5000);
    
    println!("response normal nbytes {:?}", normal_resp);
    println!("_response should takes {:?} ms", normal_resp / 5000);
    

    let start_1 = Instant::now();
    stream.write_all(& res_01).unwrap();
    stream.read_exact(& mut acknown).unwrap();
    let finis_1 = Instant::now();

    let start_2 = Instant::now();
    stream.write_all(& res_02).unwrap();
    stream.read_exact(& mut acknown).unwrap();
    let finis_2 = Instant::now();

    println!("add dbitem comp delay {:?}", t_sum);
    println!("xor parity comp delay {:?}", xor_t2 + xor_t1);
    
    println!("response xorpir delay {:?}", (finis_1 - start_1) + (finis_2 - start_2));
}



fn handle_write(hbuffer: & mut HintStorage, mut stream: TcpStream)
{
    let mut block = vec![0u8; ESIZE * 2];
    let mut raw_pos = [0u8; 2];

    // println!("\n===== new deterministic write =====\n");
    
    stream.read_exact(& mut raw_pos).expect("request write pos fail");
    stream.read_exact(& mut block).expect("request write fail");

    let counter = u16::from_be_bytes(raw_pos) as usize;
    hbuffer.write(counter, & block[.. ESIZE]);
    hbuffer.write(counter + HSIZE, & block[ESIZE ..]);
}



fn main()
{
    let listener = TcpListener::bind(SERVER_ADDRESS).expect("error binding");

    let mut storage = StoragePlus::new();
    let mut hbuffer = HintStorage::new();

    let mut switch = 0;

    loop {
        match listener.accept() {
            
            Ok((stream, _)) => 
            {
                switch += 1;
                
                if switch % 3 != 0
                {
                    println!("\n===== (Per Server Cost) Test DB: 2^{:?} entries {:?} KB", LSIZE * 2, BSIZE / 1024);
                    
                    handle_client(& mut storage, & mut hbuffer, stream);
                }
                else {
                    handle_write(& mut hbuffer, stream);
                }
            },

            Err(e) => println!("error connection: {}", e)
        }
    }
}
