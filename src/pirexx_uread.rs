
use std::convert::TryInto;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::ops::DerefMut;
use std::time::Duration;
use std::time::Instant;
use memmap::MmapMut;

mod libs;
use libs::*;


pub const KEY : [u8; 16] = [0x77; 16];


extern "C"
{
    fn set_key_and_bid(input: *const u8, size: usize, bid: u32);

    fn set_input_encryption(input: *const u8, size: usize);

    fn set_input_decryption(input: *const u8, size: usize);

    fn get_output_encryption(input: *mut u8, size: usize);

    fn get_output_decryption(input: *mut u8, size: usize);

    fn load_table();

    fn free_table();

    fn thread_encrypt();

    fn thread_decrypt();
}


pub struct Client {
    crypto: Crypto,
    item: File,
    kset: MmapMut,
    ppos: MmapMut,
    wdet: u16,
    wfile: File
}

impl Client
{
    pub fn new() -> Self
    {
        let crypto = Crypto::new();
        
        // let mut elgamal_key = File::open("ekey").expect("open fail");
        
        // let mut raw_sk = [0u8; 32];
        
        // elgamal_key.read_exact(&mut raw_sk).expect("read sk fail");

        let mut wfile = File::open("detw").expect("init detw fail");

        let mut raw_det = [0u8; 2];

        wfile.read_exact(&mut raw_det).expect("read detw fail");

        let wdet = u16::from_be_bytes(raw_det);

        let wfile = File::create("detw").expect("init detw fail");


        let item = File::create("item").expect("init item file fail");

        let key_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("kset").expect("init kset fail");

        let pos_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("ppos").expect("init ppos fail");

        let kset = unsafe { MmapMut::map_mut(& key_file).expect("map kset fail") };
        let ppos = unsafe { MmapMut::map_mut(& pos_file).expect("map ppos fail") };

        Self {crypto, item, kset, ppos, wdet, wfile}
    }

    pub fn search(& self, pk: usize, offset: B_OFFSET) -> (&[u8], Vec<u8>, Vec<u8>, usize, Duration)
    {
        let mut t_comp = Duration::from_secs(0);

        for i in 0 .. HSIZE
        {
            let key = & self.kset[i * KSIZE .. (i + 1) * KSIZE];

            let (test, t_val) = self.crypto.key_val(key, pk);
            t_comp += t_val;

            if offset == test {

                let pos = & self.ppos[i * 2 .. (i + 1) * 2];
                let pos = u16::from_be_bytes(pos.try_into().unwrap()) as usize;

                let start = Instant::now();
                let (str_a, str_b, _t_gen) = self.crypto.gen_pir(pos);
                t_comp += Instant::now() - start;

                // println!("key index {:?}", i);
                // println!("par index {:?}", pos);
                
                return (key, str_a, str_b, i, t_comp);
            }
        }

        println!("search hint fail");

        return (&[0], vec![0; 0], vec![0; 0], 0, t_comp)
    }

    pub fn newkey(& self, pk: usize, offset: B_OFFSET) -> (B_KEYSET, bool)
    {
        let mut key = Z_KEYSET;
        loop {
            self.crypto.os_random(& mut key);

            let (res, _t) = self.crypto.key_val(& key, pk);

            if offset ==  res {
                return (key, (key[0] & 1 != 0))
            }
        }
    }

    pub fn patch(& self, pk: usize, key: &[u8], away: bool) -> (TSUB, TSUB, Duration)
    {        
        let (par0, par1, zeta, t_gen) = self.crypto.gen_ppr(pk);
        
        let mut rset = vec![0u8; IV_SQRT];
        self.crypto.os_random(& mut rset);

        let start = Instant::now();

        let mut sset = self.crypto.key_set(key);
        sset[pk * ISQRT .. (pk + 1) * ISQRT].copy_from_slice(& zeta);
    
        let sub0 = (sset, par0);
        let sub1 = (rset, par1); // zeta in par1, so it goes with rset

        let finis = Instant::now();

        if away 
        {
            return (sub0, sub1, finis - start + t_gen); 
        }
        else {
            return (sub1, sub0, finis - start + t_gen);
        }
        
    }
    
    pub fn request(& self, address: &str, data_query: TSUB, refresh_query: TSUB, first_bitvec: &[u8], second_bitvec: &[u8]) -> ([Vec<u8>; 2], [Vec<u8>; 2], Vec<u8>, Vec<u8>)
    {
        let mut stream = TcpStream::connect(address).expect("stream fail");

        let list : [& [u8]; 4] = [
            & data_query.0, 
            & data_query.1, 
            & refresh_query.0, 
            & refresh_query.1
        ];

        let mut acknown = [0u8; 1];
        let mut first_bitvec_result = vec![0u8; ESIZE];
        let mut second_bitvec_result = vec![0u8; ESIZE];
        let mut dat_0 = vec![0u8; BSIZE];
        let mut dat_1 = vec![0u8; BSIZE];
        let mut new_0 = vec![0u8; BSIZE];
        let mut new_1 = vec![0u8; BSIZE];

        let start = Instant::now();
        stream.write_all(& first_bitvec).unwrap();
        stream.read_exact(& mut acknown).unwrap();
        let finis = Instant::now();
        println!("send bitvec 01 delay {:?}", finis - start);

        let start = Instant::now();
        stream.write_all(& second_bitvec).unwrap();
        stream.read_exact(& mut acknown).unwrap();
        let finis = Instant::now();
        println!("send bitvec 02 delay {:?}", finis - start);


        for i in 0 .. 4
        {
            let mut request_buffer = vec![0u8; 0];
            let length = (list[i].len() as u32).to_be_bytes();
            request_buffer.extend_from_slice(& length);
            request_buffer.extend_from_slice(list[i]);

            stream.write_all(& request_buffer).unwrap();
        }
        
        stream.read_exact(& mut dat_0).unwrap();
        stream.read_exact(& mut dat_1).unwrap();
        stream.read_exact(& mut new_0).unwrap();
        stream.read_exact(& mut new_1).unwrap();

        stream.read_exact(& mut first_bitvec_result).unwrap();
        stream.write_all(& acknown).unwrap();

        stream.read_exact(& mut second_bitvec_result).unwrap();
        stream.write_all(& acknown).unwrap();

        let finis = Instant::now();

        println!("end-to-end delay {:?}", finis - start);
    
        return ([dat_0, dat_1], [new_0, new_1], first_bitvec_result, second_bitvec_result);
    }


    pub fn parity(& self, _bid: usize, half_a: & [u8], half_b: &[u8]) -> Vec<u8>
    {
        let mut res = vec![0u8; ESIZE];
        let mut _test = vec![117u8; BSIZE];

        let start = Instant::now();

        for i in 0 .. ESIZE
        {
            res[i] = half_a[i] ^ half_b[i]
        }

        let finis = Instant::now();

        println!("parity xor recover delay {:?}", finis - start);

        unsafe {
            set_key_and_bid(KEY.as_ptr(), KEY.len(), _bid as u32);
            set_input_encryption(_test.as_ptr(), _test.len());
            thread_encrypt();
            get_output_encryption(res.as_mut_ptr(), res.len());
        }

        unsafe {
            set_key_and_bid(KEY.as_ptr(), KEY.len(), _bid as u32);
            set_input_decryption(res.as_ptr(), res.len());
            thread_decrypt();
            get_output_decryption(_test.as_mut_ptr(), _test.len());
        }

        return _test;
    }

    
    pub fn access(& mut self, x: INDX)
    {
        // prepare the queries

        let (partition_index, offset) = self.crypto.ppr_val(x);
        let (new_key, _id) = self.newkey(partition_index, offset);
        
        let (current_key, x_bitvec, y_bitvec, hint_index, time_search) = self.search(partition_index, offset);
        let (query_0, query_1, time_patch_query) = self.patch(partition_index, current_key, true); // server id is set to 1
        let (refresh_0, refresh_1, time_patch_refresh) = self.patch(partition_index, & new_key, true); // server id is neg to 0

        println!("search key + parity delay {:?}", time_search);
        println!("patch hint delay {:?}", time_patch_query + time_patch_refresh);

        // end prepare

        // prepare oblivios write

            // new key value to local

            let kset = self.kset.deref_mut();
            let new_key_storage = & mut kset[hint_index * KSIZE .. (hint_index + 1) * KSIZE];
            new_key_storage.copy_from_slice(& new_key);
    
    
            // get position for oblivious write in left buffer
    
            let counter = self.wdet as usize;
            let left_pos = & mut self.ppos[counter * 2 .. (counter + 1) * 2];
            let pos = u16::from_be_bytes(left_pos.try_into().unwrap()) as usize;
            left_pos.copy_from_slice(& self.wdet.to_be_bytes());

            let (a_bitvec, b_bitvec, time_gen) = self.crypto.gen_pir(pos);
            println!("PIR bitgen delay {:?}", time_gen);
    
            // set position for oblivious write in right buffer
    
            let righ_pos = & mut self.ppos[hint_index * 2 .. (hint_index + 1) * 2];
            righ_pos.copy_from_slice(& ((counter + HSIZE) as u16).to_be_bytes());


        // end prepare
    
        let (q0_result, r0_result, x_parity, a_parity) = self.request(SERVER_ADDRESS, query_0, refresh_0, & x_bitvec, & a_bitvec);
        let (q1_result, r1_result, y_parity, b_parity) = self.request(SERVER_ADDRESS, query_1, refresh_1, & y_bitvec, & b_bitvec);


        let current_parity = self.parity(hint_index, & x_parity, & y_parity);
        let rewrite_parity = self.parity(counter, & a_parity, & b_parity);


        let (data_item, t_rec) = self.recover_dbitem([& q0_result[0], & q0_result[1], & current_parity, & q1_result[1]]);
        let (refresh_parity, t_ref) = self.refresh_parity([& r0_result[0], & r0_result[1], & data_item, & r1_result[1]]);

        println!("item recover delay {:?}", t_rec + t_ref);
        self.rewrite(rewrite_parity, counter, refresh_parity, hint_index);


        let view = format!("{:?}", data_item);
        self.item.write_all(view.as_bytes()).unwrap();
    }

    pub fn rewrite(& mut self, _rewrite_parity: Vec<u8>, _counter: usize, _refresh_parity: Vec<u8>, _hint_index: usize)
    {   
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("stream fail");

        let mut _enc_left = vec![0u8; ESIZE];
        let mut _enc_righ = vec![0u8; ESIZE];

        // unsafe {
        //     set_key_and_bid(KEY.as_ptr(), KEY.len(), counter as u32);
        //     set_input_encryption(rewrite_parity.as_ptr(), rewrite_parity.len());
        //     thread_encrypt();
        //     get_output_encryption(enc_left.as_mut_ptr(), enc_left.len());
        // }

        // // need additional code review

        // unsafe {
        //     set_key_and_bid(KEY.as_ptr(), KEY.len(), hint_index as u32);
        //     set_input_encryption(refresh_parity.as_ptr(), refresh_parity.len());
        //     thread_encrypt();
        //     get_output_encryption(enc_righ.as_mut_ptr(), enc_righ.len());
        // }
        
        let write_data = [self.wdet.to_be_bytes().to_vec(), _enc_left, _enc_righ].concat();
        stream.write_all(& write_data).expect("oblivious write fail");


        self.wdet = (((self.wdet + 1) as usize) % HSIZE) as u16;
        self.wfile.write_all(& self.wdet.to_be_bytes()).expect("next pos");
    }

    pub fn recover_dbitem(& self, input: [& [u8]; 4]) -> (Vec<u8>, Duration)
    {
        // println!("input {:?}", input);

        let mut block = vec![0u8; BSIZE];
        let mut regis : [PINT; BSIZE / MSIZE] = [0; BSIZE / MSIZE];

        let mut t_comp = Duration::from_secs(0);
        
        for i in [2, 3]
        {
            for (j, chunk) in input[i].chunks(MSIZE).enumerate()
            {
                let chunk : [u8; MSIZE] = chunk.try_into().unwrap();
                let temp = PINT::from_be_bytes(chunk);

                let start = Instant::now();
                regis[j] += temp;
                t_comp += Instant::now() - start;
            }
        }

        for i in [0, 1]
        {
            for (j, chunk) in input[i].chunks(MSIZE).enumerate()
            {
                let chunk : [u8; MSIZE] = chunk.try_into().unwrap();
                let temp = PINT::from_be_bytes(chunk);

                let start = Instant::now();
                regis[j] -= temp;
                t_comp += Instant::now() - start;
            }
        }

        for (i, chunk) in block.chunks_exact_mut(MSIZE).enumerate()
        {
            chunk.copy_from_slice(& regis[i].to_be_bytes());
        }

        return (block, t_comp);
    }

    pub fn refresh_parity(& self, input: [& [u8]; 4]) -> (Vec<u8>, Duration)
    {
        // println!("input {:?}", input);

        let mut block = vec![0u8; BSIZE];
        let mut regis : [PINT; BSIZE / MSIZE] = [0; BSIZE / MSIZE];
        
        let mut t_comp = Duration::from_secs(0);
        
        for i in [0, 1, 2]
        {
            for (j, chunk) in input[i].chunks(MSIZE).enumerate()
            {
                let chunk : [u8; MSIZE] = chunk.try_into().unwrap();
                let temp = PINT::from_be_bytes(chunk);

                let start = Instant::now();
                regis[j] += temp;
                t_comp += Instant::now() - start;
            }
        }

        for i in [3]
        {
            for (j, chunk) in input[i].chunks(MSIZE).enumerate()
            {
                let chunk : [u8; MSIZE] = chunk.try_into().unwrap();
                let temp = PINT::from_be_bytes(chunk);

                let start = Instant::now();
                regis[j] -= temp;
                t_comp += Instant::now() - start;
            }
        }

        for (i, chunk) in block.chunks_exact_mut(MSIZE).enumerate()
        {
            chunk.copy_from_slice(& regis[i].to_be_bytes());
        }

        return (block, t_comp);
    }
}

fn main()
{
    unsafe {load_table()}

    println!("Test DB: 2^{:?} entries {:?} KB", LSIZE * 2, BSIZE / 1024);

    let mut client = Client::new();

    let index = (12482 % NSIZE) as INDX;
    let n_test = 1;

    for _ in 0 .. n_test
    {
        client.access(index);
    }


    unsafe {free_table()}
}
