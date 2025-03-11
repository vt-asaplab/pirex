
# PIREX: CLIENT-EFFICIENT ONLINE-OFFLINE PIR

Note: This project was last updated in: March-11-2025

WARNING: This is an academic proof-of-concept prototype and has not received careful code review. This implementation is NOT ready for production use.


## HARDWARE

The code can be run on any hardware.

We have also configured server computation to be easily accelarated using `x86_64` SIMD instruction



## DIRECTORY EXPLAIN

From the root, we have three directories:

1. `secp256k1` is where we store our customized `secp256k1` library.

2. `src` is where we store the codes for our proposed schemes.

3. `utils` is where we store helper data and cpp function.





## SETUP RUST ENVIRONMENT

1. Install Rust: https://www.rust-lang.org/tools/install
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install nightly release
```
rustup toolchain install nightly-2023-09-24
```

3. Switch to nightly release
```
rustup default nightly-2023-09-24
```



## COMPILE DEPENDENCY

For `pirex+`, we have customized `secp256k1` library and put
under `secp256k1` folder.

Before being able to compile our `secp256k1`, you may need to install some building library as follow:

```
sudo apt install build-essential
apt install autoconf automake libtool
```

After check the above requirements, you need to move inside `secp256k1` folder to actuall compile it.

```
./autogen.sh
./configure
make
```

If successfully, we will see the static binary `libsecp256k1.a`
under subfolder `secp256k1/.libs`, which will be used
when we build our main source code in rust.

Our code will need a precomputed discrete log table when using `libsecp256k1.a`.

To save time, you can download it from: https://www.dropbox.com/scl/fi/09dufxsm3qc4nnemdqgpv/dlp.bin?rlkey=rtcq1banfpl0cehdd2oa1b7zy&st=rfhznwre&dl=0

Name it as `dlp.bin` and put it into `utils` folder in the root.





## CONFIGURE PARAMETERS

1. Change the `SERVER_ADDRESS` inside `src/libs.rs`


2. Change the blocksize (4KiB, 64KiB, 256KiB) and database size ($2^{12}$ to $2^{28}$)

    Our code can work with any blocksize as long as the bytes' amount is divisible by 64.

    For example: 4KiB = 4096 bytes and 4096 / 64 = 64. Thus, to test with 4KiB and $2^{22}$ records:

```
python3 config.py 64 22
```

3. Compile the configured source code

```
cargo build --release
```

If everything is correctly builded and linked, you should see the following output:

```
warning: secp256k1 library path: <your_home_path>/pirex/secp256k1/.libs
```






## OFFLINE PHASE

(For `pirex+`, replace the filename `pirex` with `pirexx`)

1. Run server code
```
./target/release/pirex_sprep
```


2. Run client code
```
./target/release/pirex_uprep
```


## ONLINE PHASE

(For `pirex+`, replace the filename `pirex` with `pirexx`)

1. Run server code
```
./target/release/pirex_sread
```


2. Run client code
```
./target/release/pirex_uread
```
