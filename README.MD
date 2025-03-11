
# PIREX: CLIENT-EFFICIENT ONLINE-OFFLINE PIR

Note: This project was last updated in: Dec-21-2024

WARNING: This is an academic proof-of-concept prototype and has not received careful code review. This implementation is NOT ready for production use.


## HARDWARE

1. Client: `aarch64` (Linux)

2. Server: `x86_64` (Linux)


## SETUP

1. Install Rust: https://www.rust-lang.org/tools/install


```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```


2. Install nightly release


```
$ rustup toolchain install nightly-2023-09-24
```


3. Switch to nightly release
```
rustup default nightly-2023-09-24
```


## CONFIG

1. Change the `SERVER_ADDRESS` inside `src/libs.rs`


2. Change the blocksize (4KiB, 64KiB, 256KiB) and database size ($2^{12}$ to $2^{28}$)

    Our code can work with any blocksize as long as the bytes' amount is divisible by 64.

    For example: 4KiB = 4096 bytes and 4096 / 64 = 64. Thus, to test with 4KiB and $2^{22}$ records:

```
python3 config.py 64 22
```







## COMPILE

0. Download the precomputed `dlp.bin` table from: https://www.dropbox.com/scl/fi/09dufxsm3qc4nnemdqgpv/dlp.bin?rlkey=rtcq1banfpl0cehdd2oa1b7zy&st=rfhznwre&dl=0

1. Move `dlp.bin` into `utils` folder

2. Change directory to compile the secp256k1 library: `cd secp256k1`

3. Compile the library:

```
./autogen.sh
./configure
make
```

4. Return to the project root: `cd ..`

5. Compile the configured source code

```
cargo build --release
```

6. Note that cargo will use static library functions from libsecp256k1 (step 3 above) to link  during the compilation, as configured inside `src/build.rs`. Only the client side
needs libsecp256k1 so this should work if the hardware requirement is satisfied.





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
