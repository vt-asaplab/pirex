
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

pub struct Client {
    crypto: Crypto,
    item: File,
    kset: MmapMut,
    hint: MmapMut,
}

impl Client
{
    pub fn new() -> Self
    {
        let crypto = Crypto::new();

        let item = File::create("item").expect("init kset fail");

        let key_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("kset").expect("init kset fail");

        let par_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("hint").expect("init hint fail");
    
        let kset = unsafe { MmapMut::map_mut(& key_file).expect("map kset fail") };
        let hint = unsafe { MmapMut::map_mut(& par_file).expect("map hint fail") };

        Self {crypto, item, kset, hint}
    }

    pub fn search(& self, pk: usize, offset: B_OFFSET) -> (&[u8], &[u8], usize, Duration)
    {
        let mut total = Duration::from_secs(0);

        for i in 0 .. HSIZE
        {
            let key = & self.kset[i * KSIZE .. (i + 1) * KSIZE];
            let par = & self.hint[i * BSIZE .. (i + 1) * BSIZE];

            let (res, t_val) = self.crypto.key_val(key, pk);
    
            if offset == res {
                return (key, par, i, total)
            }

            total += t_val;
        }
        println!("search hint fail");

        return (&[0], &[0], 0, total)
    }

    pub fn newkey(& self, pk: usize, offset: B_OFFSET) -> B_KEYSET
    {
        let mut key = Z_KEYSET;
        loop {
            self.crypto.os_random(& mut key);

            let (res, _t_val) = self.crypto.key_val(& key, pk);

            if offset == res {
                return key
            }
            // simplified, can be faster with shifting PRS
        }
    }

    pub fn patch(& self, pk: usize, key: &[u8]) -> (B_OFFSET, TSUB, TSUB, Duration)
    {        
        let mut rset = vec![0u8; IV_SQRT];
        self.crypto.os_random(& mut rset);

        let start = Instant::now();

        println!("stop here");
        
        let (par0, par1, zeta, _) = self.crypto.gen_ppr(pk);
        let mut sset = self.crypto.key_set(key);
        sset[pk * ISQRT .. (pk + 1) * ISQRT].copy_from_slice(& zeta);

        let finis = Instant::now();
    
        let sub0 = (sset, par0);
        let sub1 = (rset, par1);
    
        return (zeta, sub0, sub1, finis - start);
    }
    
    pub fn request(& self, qa: TSUB, qb: TSUB) -> (Duration, [Vec<u8>; 4], usize)
    {
        let mut stream = TcpStream::connect(SERVER_ADDRESS).expect("stream fail");

        let list : [& [u8]; 4] = [& qa.0, & qa.1, & qb.0, & qb.1];
        
        let mut buffer = vec![0u8; 0];
        let mut response = vec![0u8; 4 * BSIZE];
        let mut acknown = [0u8; 1];

        for i in 0 .. 4
        {
            let length = (list[i].len() as u32).to_be_bytes();
            buffer.extend_from_slice(& length);
            buffer.extend_from_slice(list[i]);
        }

        // println!("outbound band in nbytes: {:?}", buffer.len());

        stream.write_all(& (buffer.len() as u32).to_be_bytes()).unwrap();
        let start = Instant::now();
        stream.write_all(& buffer).unwrap();
        stream.read_exact(& mut acknown).unwrap();
        let finis = Instant::now();

        stream.read_exact(& mut response).unwrap();
        stream.write(& acknown).unwrap();

        let mut r_query = [
            vec![0u8; BSIZE],
            vec![0u8; BSIZE],
            vec![0u8; BSIZE],
            vec![0u8; BSIZE],
        ];

        // println!("inbound band in nbytes: {:?}", response.len());

        for i in 0 .. 4
        {
            let val = & response[i * BSIZE .. (i + 1) * BSIZE];
            r_query[i].copy_from_slice(val);
        }
    
        return (finis - start, r_query, buffer.len() + response.len());
    }
    
    pub fn access(& mut self, x: INDX) -> (Duration, Duration, usize)
    {    
        let (pk, offset) = self.crypto.ppr_val(x);
        let (sk, px, _hi, tx_search) = self.search(pk, offset);
        
        let (zeta, q0, q1, pat_t1) = self.patch(pk, sk);

        let zeta = parse_index_2((pk as INDX) << LSIZE, & zeta) as INDX;
        let (z_pk, z_offset) = self.crypto.ppr_val(zeta);
        let (_z_sk, _z_px, _hz, tz_search) = self.search(z_pk, z_offset);
        
        // let (r0, r1, pat_t2) = self.patch(pk, & nk);
        // println!("query {pk}: {:?}", offset);
        // println!("hint: {hi}");
    
        let (t_band, mut r_query, band_size) = self.request(q0, q1);
        
        r_query[2].copy_from_slice(px);
        let (dbitem, rec_t1) = self.recover(r_query);


        let view = format!("{:?}", dbitem);
        self.item.write_all(view.as_bytes()).unwrap();
        // self.rewrite(hi, nk, parity);

        let t_comp = (tx_search + tz_search) + (rec_t1 * 2) + (pat_t1);

        // println!("outbound delay: {:?}", t_band);
        // println!("client search delay: {:?}", tx_search + tz_search);
        // println!("client patch delay: {:?}", pat_t1);
        // println!("client recover delay: {:?}", rec_t1 * 2);

        return (t_comp, t_band, band_size);
    }

    pub fn rewrite(& mut self, hi: usize, nk: B_KEYSET, np: Vec<u8>)
    {
        let kset = self.kset.deref_mut();
        let hint = self.hint.deref_mut();

        let nkey = & mut kset[hi * KSIZE .. (hi + 1) * KSIZE];
        let npar = & mut hint[hi * BSIZE .. (hi + 1) * BSIZE];

        nkey.copy_from_slice(& nk);
        npar.copy_from_slice(& np);
    }

    pub fn recover(& self, input: [Vec<u8>; 4]) -> (Vec<u8>, Duration)
    {
        let mut block = vec![0u8; BSIZE];
        let mut regis = [LANE::splat(0); BUNIT];

        let start = Instant::now();
        
        for i in 0 .. 4
        {
            for (j, chunk) in input[i].chunks(USIZE).enumerate()
            {
                regis[j] ^= LANE::from_slice_unaligned(chunk);
            }
        }
        let finis = Instant::now();

        for (i, chunk) in block.chunks_exact_mut(USIZE).enumerate()
        {
            regis[i].write_to_slice_unaligned(chunk);
        }

        return (block, finis - start);
    }
}

fn main()
{
    let mut client = Client::new();
    let mut t_comp = Duration::from_secs(0);
    let mut t_band = Duration::from_secs(0);
    let mut t_size: usize = 0;

    let index = (12482 % NSIZE) as INDX;
    let n_test = 20;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("results/pirex_client_online.txt").unwrap();

    println!("\n===== PIREX - Test DB: 2^{:?} entries {:?} KB", LSIZE * 2, BSIZE / 1024);
    file.write_all(format!("\n===== PIREX - Test DB: 2^{:?} entries {:?} KB \n", LSIZE * 2, BSIZE / 1024).as_bytes()).unwrap();

    for _ in 0 .. n_test
    {
        let (i_comp, i_band, i_size) = client.access(index);

        t_comp += i_comp;
        t_band += i_band;
        t_size += i_size;
    }

    file.write_all(format!("client computation elapse {:?} \n", t_comp / n_test).as_bytes()).unwrap();
    file.write_all(format!("client request elapse {:?} \n", t_band / n_test).as_bytes()).unwrap();
    file.write_all(format!("total bandwidth nbytes {:?} \n", t_size / (n_test as usize)).as_bytes()).unwrap();
}
