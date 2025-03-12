
import subprocess
import time
import sys


small_case = [
    "python3 config.py 64 18",
    "python3 config.py 64 20",
    "python3 config.py 1024 14",
    "python3 config.py 1024 16",
    "python3 config.py 4096 12",
    "python3 config.py 4096 14",
]

medium_case = [
    "python3 config.py 64 22",
    "python3 config.py 64 24",
    "python3 config.py 1024 18",
    "python3 config.py 1024 20",
    "python3 config.py 4096 16",
    "python3 config.py 4096 18",
]

large_case = [
    "python3 config.py 64 26",
    "python3 config.py 64 28",
    "python3 config.py 1024 22",
    "python3 config.py 1024 24",
    "python3 config.py 4096 20",
    "python3 config.py 4096 22",
]

build = "cargo build --release"

pirex_server = "./target/release/pirex_sread"
pirex_client = "./target/release/pirex_uread"

find_port = "lsof -t -i :8111"


def check(case):

    for test in case:
        subprocess.run(test, shell=True, check=True)
        subprocess.run(build, shell=True, check=True)
        
        process = subprocess.Popen(pirex_server, shell=True)
        subprocess.run(pirex_client, shell=True, check=True)

        PID = subprocess.run(find_port, capture_output=True, shell=True, text=True).stdout.strip()
        subprocess.run(f"kill -9 {PID}", shell=True, check=True)



if len(sys.argv) > 1:

    inp = sys.argv[1]
    
    if inp == "small": check(small_case)
    
    if inp == "medium": check(medium_case)
    
    if inp == "large": check(large_case)

