

# Artifact

Paper title: **Client-Efficient Online-Offline Private Information Retrieval**

Artifacts HotCRP Id: **#13**

Requested Badge: **Available**, **Functional**, or **Reproduced**

Note: This project was last updated in: March-11-2025

Warning: This is an academic proof-of-concept prototype and has not received careful code review. This implementation is NOT ready for production use.

<br/>


## Description

This artifact contains source code to our two proposed scheme Pirex/Pirex+


### Security/Privacy Issues and Ethical Concerns (All badges)
Our implementation uses common `rust` modules from standard `crates.io`
which is automatically managed by `cargo` package manager. In addition,
we use standard cryptographic library `libsecp256k1` from `bitcoin-core` 
which only requires `libtool`, `automake`, `build-essential` for compile 
process and can be easily installed via package management tools (e.g. 
apt on Ubuntu). The execution of this artifact does not pose any risks 
to the evaluators’ system security, data privacy, or ethical concerns.


<br/>

## Basic Requirements (Only for Functional and Reproduced badges)

We describe the minimal hardware and software requirements for our artifact 
and estimate the compute time and storage required to run the artifact.

<br/>

### Hardware Requirements
No specific hardware is required to execute this artifact.

A laptop with `x86_64` bit architecture with at least 8-core CPU, 16GB RAM and 
1TB storage can already satisfy our hardware requirements. 

In adddition, our artifact can automatically make use RAM resource (upto 1TB) 
to accelerate the server performance.


### Network Expectation

Our artifact supports network connection between client and server through
TCP stream, which can be adjusted for both remote and local testing.

Regardless of the testing environment, we expect the network bandwidth will
be around 40~45 Mbps to match our main results and claims.



<br/>

### Software Requirements

All required `rust` modules and `libsecp256k1` source codes have
already been incorporated into the current artifact.

<br/>


### Estimated Time and Storage Consumption

This artifact contains the implementation of two schemes Pirex/Pirex+
Each scheme has two phases to be evaluated: (1) Offline, (2) Online.

The evaluation of the offline phase per scheme can take about 01 human days.

The evaluation of the online phase per scheme can take about 05 human hours.

Since the offline phase only affect the correctness of our scheme, 
we have provided instructions so that the offline phase can be skipped
but still allow the full efficiency evaluation of the online phase.

The total required storage size for full evaluation is under 1TB 
(max database setting in our schemes).

We recommend running the first 04 test cases of our artifact, which
only consist 04 online phases for each scheme, and only take upto 02
human hours with 100GB storage in corresponding.

<br/>





## Environment 

In this section, we describe the component of our artifact and how 
to verify that everything is set up correctly.

<br/>

### Accessibility

Our artifact is entirely hosted on this github repository.


### Directory Structure

From the root, we have three directories:
1. `results` is where we store all performace breakdown results.
2. `secp256k1` is where we store our customized `libsecp256k1` library.
3. `src` is where we store the codes for our proposed schemes.
4. `utils` is where we store helper data and cpp function.

<br/>

### Setup Environment

To configure the correct `rust` version:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustup toolchain install nightly-2023-09-24
rustup default nightly-2023-09-24
```

<br/>


### Install Dependency

For `pirex+`, we have customized `libsecp256k1` library and put
under `secp256k1` folder.

Before being able to compile our `libsecp256k1`, you may need to 
install some building libraries as follow:

```
sudo apt install build-essential
sudo apt install autoconf automake libtool
```

After checking all requirements, move into `secp256k1` for compilation.

```
./autogen.sh
./configure
make
```

If successfully, we will see the static binary `libsecp256k1.a`
under subfolder `secp256k1/.libs`, which will be used
when we build our main source code in rust.

`libsecp256k1.a` will need a precomputed discrete log table.

To save time, you can download it from: https://www.dropbox.com/scl/fi/09dufxsm3qc4nnemdqgpv/dlp.bin?rlkey=rtcq1banfpl0cehdd2oa1b7zy&st=rfhznwre&dl=1

```
curl -L -o dlp.bin <the_above_link>
```

Move `dlp.bin` into `utils` folder in the root.


<br/>


### Configure & Build

1. Edit `SERVER_ADDRESS` inside `src/libs.rs` (We use localhost as default)

2. Check if environment is configured correctly for automate compilation:

```
python3 config.py 64 22
cargo build --release
```

If the artifact is correctly builded, you should see the following output:

```
warning: secp256k1 library path: <your_home_path>/pirex/secp256k1/.libs
```

The above commands are the first step to help us to test the performance
on a database of $2^{22}$ records, each of size 04KiB. 

Our artifact divide blocksize to multiple chunks of 64 bytes. Thus:
- 04KiB = 4096 bytes --> input `64` chunks
- 64KiB = 65536 bytes --> input `1024` chunks
- 256KiB = 262144 bytes --> input `4096` chunks

    






## Artifact Evaluation

In this section, we include all the steps required to evaluate our artifact's functionality and validate our paper's key results and claims.

### Main Results and Claims
- Constant client inbound bandwidth cost (`pirex`, `pirex+`)
- Low client online end-to-end delay (`pirex`, `pirex+`)
- Low client storage cost (`pirex+`)

<br/>


### Performance Testcases

To obtain the results that support above claims,
we will measure the performance of our two schemes:

Under 03 types of blocksize
- 04KiB (`64` chunks of 64 bytes)
- 64KiB (`1024` chunks of 64 bytes)
- 256KiB (`4096` chunks of 64 bytes)

With 03 range scale (pow of two)
- [`18`, `20`, `22`, `24`, `26`, `28`]
- [`14`, `16`, `18`, `20`, `22`, `24`]
- [`12`, `14`, `16`, `18`, `20`, `22`]

This produce the following testcase:
- Small --> Database Size 1GiB - 4GiB
- Medium --> Database Size 16GiB - 64GiB
- Large --> Database Size 256GiB - 1024GiB

We have provided a `control.py` script to automate
this testing process with input: `small`, `medium`, `large`.
This script can help us to skip the offline phase and
directly measure breakdown cost of the online phase.

<br/>



### Experiment Procedure

We recommend start with `small` then `medium` testcase.
We will first need to randomize the client and server
data structure to simulate (without correctness) using 
`src/helper.rs`. We can run a helper code as follow:
```
python3 config.py 64 24
cargo build --release 
./target/release/helper
```

This will create data structures that are enough for us
to first test the performace of `small`, `medium` cases.

We can then run the automate script `control.py` as follow:
```
python3 control.py pirex small
python3 control.py pirex medium
```

All breakdown cost will be logged inside `results` folder:
- Total client computation delay
- Total server computation delay
- In/Out bandwidth cost as number of bytes
- In/Out bandwidth delay (w.r.t the expected 40 Mbps)
- In/Out bandwidth delay (w.r.t your local environment)

Since `large` testcase can consume 1TB of storage disk 
to randomly create a database for testing, we recommend
checking your storage space first. If everything is ok,
run as follow to setup data structure for `large` case:

```
python3 config.py 64 28
cargo build --release 
./target/release/helper
python3 control.py pirex large
```


#### Main Result 1: Constant Inbound Bandwidth
- Figure 8


#### Main Result 2: Low Client Online End-End Delay
- Server computation delay + Client computation delay
- Figure 10 + 11 + 12



#### Main Result 2: Low Client Storage Cost
- Storage Cost (Figure 14)








### More Self Evaluation

#### OFFLINE PHASE

(For `pirex+`, replace the filename `pirex` with `pirexx`)

1. Run server code in current terminal
```
./target/release/pirex_sprep
```

2. Run client code in another terminal
```
./target/release/pirex_uprep
```


#### ONLINE PHASE

(For `pirex+`, replace the filename `pirex` with `pirexx`)

1. Run server code in current terminal
```
./target/release/pirex_sread
```


2. Run client code in another terminal
```
./target/release/pirex_uread
```
