
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

pirexx_server = "./target/release/pirexx_sread"
pirexx_client = "./target/release/pirexx_uread"

find_port = "lsof -t -i :8111"


def pirex_test(case):

    for test in case:
        subprocess.run(test, shell=True, check=True)
        subprocess.run(build, shell=True, check=True)
        
        process = subprocess.Popen(pirex_server, shell=True)
        subprocess.run(pirex_client, shell=True, check=True)

        PID = subprocess.run(find_port, capture_output=True, shell=True, text=True).stdout.strip()
        subprocess.run(f"kill -9 {PID}", shell=True, check=True)


def pirexx_test(case):

    for test in case:
        subprocess.run(test, shell=True, check=True)
        subprocess.run(build, shell=True, check=True)
        
        server = subprocess.Popen(pirexx_server, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        client = subprocess.run(pirexx_client, capture_output=True, shell=True, check=True)

        PID = subprocess.run(find_port, capture_output=True, shell=True, text=True).stdout.strip()
        subprocess.run(f"kill -9 {PID}", shell=True, check=True)

        client_out = client.stdout.strip()
        server_out, stderr = server.communicate() 

        with open("results/pirexx_client_online.txt", "ab") as file:
            file.write(client_out)

        with open("results/pirexx_server_online.txt", "ab") as file:
            file.write(server_out)


if len(sys.argv) > 1:

    sch = sys.argv[1]
    
    inp = sys.argv[2]

    if sch == "pirex":
    
        if inp == "small": pirex_test(small_case)
        
        if inp == "medium": pirex_test(medium_case)
        
        if inp == "large": pirex_test(large_case)
    
    if sch == "pirexx":
    
        if inp == "small": pirexx_test(small_case)
        
        if inp == "medium": pirexx_test(medium_case)
        
        if inp == "large": pirexx_test(large_case)

