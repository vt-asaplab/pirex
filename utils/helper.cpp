

#include <omp.h> // for omp
#include <cstdint> // for uint8_t
#include <cstddef> // for size_t
#include <cstdio>


#ifdef __x86_64__

#include <immintrin.h>

const size_t chunk_size = sizeof(__m256i); // 256 bits = 64 bytes

extern "C" {
    void xor_byte_arrays(uint8_t *a, uint8_t *const b, size_t size)
    {        
        #pragma omp parallel for num_threads(12)
        for (size_t i = 0; i < size / chunk_size; i++)
        {
            __m256i x1 = _mm256_loadu_si256((__m256i*) &a[i * chunk_size]);
            // printf("x1 pass iter: %zu\n", i);
            
            __m256i x2 = _mm256_loadu_si256((__m256i*) &b[i * chunk_size]);
            // printf("x2 pass iter: %zu\n", i);

            __m256i x3 = _mm256_xor_si256(x1, x2);
            // printf("x3 pass iter: %zu\n", i);
            
            _mm256_storeu_si256((__m256i*) &a[i * chunk_size], x3);
            // printf("assign pass iter: %zu\n", i);
        }
    }
}

#else

typedef unsigned long long int TYPE_REGISTER;

TYPE_REGISTER *x1,*x2;

extern "C" {
    void xor_byte_arrays(uint8_t *a, uint8_t *const b, size_t size) {
        
        // #pragma omp parallel for num_threads(2)
        for (size_t i = 0; i < size; i += sizeof(TYPE_REGISTER))
        {
            x1 = (TYPE_REGISTER*) &a[i];
            x2 = (TYPE_REGISTER*) &b[i];
            *x1 ^= *x2;
        }
    }
}

typedef uint16_t TYPE_MODULUS;

TYPE_MODULUS *a1,*a2;

extern "C" {
    void add_byte_arrays(uint8_t *a, uint8_t *const b, size_t size) {
        
        for (size_t i = 0; i < size; i += sizeof(TYPE_MODULUS))
        {
            a1 = (TYPE_MODULUS*) &a[i];
            a2 = (TYPE_MODULUS*) &b[i];
            *a1 += *a2;
        }
    }
}

extern "C" {
    void sub_byte_arrays(uint8_t *a, uint8_t *const b, size_t size) {
        
        for (size_t i = 0; i < size; i += sizeof(TYPE_MODULUS))
        {
            a1 = (TYPE_MODULUS*) &a[i];
            a2 = (TYPE_MODULUS*) &b[i];
            *a1 -= *a2;
        }
    }
}

#endif



#include <numeric>
#include <algorithm>
#include <cstdio>
#include <cstdint>
#include <iostream>
#include <chrono>
#include <cstring>
#include <vector>
#include <thread>
#include <sched.h>

#include "../secp256k1/include/secp256k1.h"
#include "random.h"


using namespace std;
using namespace std::chrono;

#define TABLE_LOG "utils/dlp.bin"



string hex_bytes(unsigned char* data, size_t size)
{
    char digit[3];
    string result;

    for (size_t i = 0; i < size; i++)
    {
        sprintf(digit, "%02x", data[i]);
        result += string(digit);
    }
    
    return result;
}

struct HashMap *TABLE;

const size_t KEY_LEN = 16;
const size_t INP_LEN = 04;
const size_t ENC_LEN = 33;
const size_t N_CHUNK = 65536; // script auto change this
const size_t THREAD_NUM = 8;
const uint32_t THREAD_SIZE = N_CHUNK / THREAD_NUM;

const size_t INP_FULLSIZE = INP_LEN * N_CHUNK;
const size_t ENC_FULLSIZE = ENC_LEN * N_CHUNK;

uint8_t KEY[KEY_LEN];
uint8_t INP[INP_LEN * N_CHUNK];
uint8_t ENC[ENC_LEN * N_CHUNK];
uint8_t DEC[INP_LEN * N_CHUNK];
uint32_t BLOCK_ID = 777;




void encrypter(uint32_t start, vector<long int>& rtimes, size_t index)
{
    secp256k1_context *CTX = secp256k1_context_create(SECP256K1_CONTEXT_NONE);
    
    uint8_t BLIND[32] = {};
    memcpy(&BLIND[0], KEY, KEY_LEN);
    memcpy(&BLIND[20], &BLOCK_ID, sizeof(BLOCK_ID));

    uint32_t finis = start + THREAD_SIZE;
    int good = 1;
    auto head = high_resolution_clock::now();

    for (uint32_t counter = start; counter < finis; counter++)
    {
        memcpy(&BLIND[28], &counter, sizeof(counter));
        good = secp256k1_elgamal_encryption(CTX, &INP[counter * INP_LEN], INP_LEN, BLIND, 32, &ENC[counter * ENC_LEN], ENC_LEN);
    }

    auto tail = high_resolution_clock::now();
    auto cpu_time = duration_cast<milliseconds>(tail - head).count();

    if (!good) printf("encrypter %d is not good\n", sched_getcpu());

    // printf("encrypt range (%d - %d) takes %ld millis\n", start, finis, cpu_time);

    rtimes[index] = cpu_time;
}



void decrypter(uint32_t start, vector<long int>& rtimes, size_t index)
{
    // int cpu = sched_getcpu();
    // printf("current CPU ID: %d \n", cpu);

    secp256k1_context *CTX = secp256k1_context_create(SECP256K1_CONTEXT_NONE);

    uint8_t BLIND[32] = {};
    memcpy(&BLIND[0], KEY, KEY_LEN);
    memcpy(&BLIND[20], &BLOCK_ID, sizeof(BLOCK_ID));

    uint32_t finis = start + THREAD_SIZE;
    int good = 1;
    auto head = high_resolution_clock::now();

    for (uint32_t counter = start; counter < finis; counter++)
    {
        memcpy(&BLIND[28], &counter, sizeof(counter));
        good = secp256k1_elgamal_decryption(CTX, TABLE, &ENC[counter * ENC_LEN], ENC_LEN, BLIND, 32, &DEC[counter * INP_LEN], INP_LEN, BABY_RANGE);
        reverse(&DEC[counter * INP_LEN], &DEC[(counter + 1) * INP_LEN]);
    }

    auto tail = high_resolution_clock::now();
    auto cpu_time = duration_cast<milliseconds>(tail - head).count();

    if (!good) printf("decrypter %d is not good\n", sched_getcpu());

    // printf("decrypt range (%d - %d) takes %ld millis\n", start, finis, cpu_time);

    rtimes[index] = cpu_time;
}




extern "C"
{
    void thread_decrypt()
    {
        vector<thread> threads;
        uint32_t starts[THREAD_NUM];
        vector<long int> rtimes(THREAD_NUM);

        for (uint32_t i = 0; i < THREAD_NUM; ++i)
        {
            starts[i] = i * THREAD_SIZE;
            threads.emplace_back(decrypter, starts[i], ref(rtimes), i);
        }

        for (auto &thread : threads) {
            thread.join();
        }

        long int total = 0;

        for (auto &res : rtimes) {
            total += res;
        }

        printf("client decrypt delay %ldms \n", total / THREAD_NUM);
        fflush(stdout);
    }


    void thread_encrypt()
    {
        vector<thread> threads;
        uint32_t starts[THREAD_NUM];
        vector<long int> rtimes(THREAD_NUM);

        for (uint32_t i = 0; i < THREAD_NUM; ++i)
        {
            starts[i] = i * THREAD_SIZE;
            threads.emplace_back(encrypter, starts[i], ref(rtimes), i);
        }

        for (auto &thread : threads) {
            thread.join();
        }

        long int total = 0;

        for (auto &res : rtimes) {
            total += res;
        }

        printf("client encrypt delay %ldms \n", total / THREAD_NUM);
        fflush(stdout);
    }


    void set_input_encryption(uint8_t *input, size_t inputlen)
    {
        if (inputlen != sizeof(INP))
        {
            printf("incorrect encryption input\n");
            return;
        }

        memset(INP, 0, sizeof(INP));
        memcpy(INP, input, inputlen);
    }

    void set_input_decryption(uint8_t *input, size_t inputlen)
    {
        if (inputlen != sizeof(ENC))
        {
            printf("incorrect decryption input\n");
            return;
        }

        memset(ENC, 0, sizeof(ENC));
        memcpy(ENC, input, inputlen);
    }


    void get_output_encryption(uint8_t *input, size_t inputlen)
    {
        if (inputlen != sizeof(ENC))
        {
            printf("incorrect encryption output\n");
            return;
        }

        memcpy(input, ENC, inputlen);
    }

    void get_output_decryption(uint8_t *input, size_t inputlen)
    {
        if (inputlen != sizeof(INP))
        {
            printf("incorrect decryption input\n");
            return;
        }

        memcpy(input, DEC, inputlen);
    }


    void free_table()
    {
        free(TABLE);
    }


    void set_key_and_bid(uint8_t *input, size_t inputlen, uint32_t bid)
    {
        BLOCK_ID = bid;

        if (inputlen != KEY_LEN)
        {
            printf("cannot set key\n");
            return;
        }

        memcpy(KEY, input, inputlen);
    }

    void init_table()
    {
        secp256k1_context *CTX = secp256k1_context_create(SECP256K1_CONTEXT_NONE);

        struct HashMapPre* TABLE;

        TABLE = (struct HashMapPre *)malloc(sizeof(struct HashMapPre));

        printf("start init\n");

        hashmap_init(TABLE);

        printf("finish init\n");

        int good = secp256k1_build_discrete_log(CTX, TABLE);

        if (good) printf("build table in test ok\n");

        FILE *file = fopen("test.bin", "wb");

        int arr[11] = {0};

        for (uint32_t i = 0; i < HASHMAP_SIZE; i++)
        {
            uint8_t length = TABLE->buckets[i].length;

            fwrite(&length, sizeof(length), 1, file);

            for (uint8_t j = 0; j < length; j++)
            {
                uint64_t x = TABLE->buckets[i].point[j];
                uint32_t a = TABLE->buckets[i].array[j];;

                fwrite(&x, sizeof(x), 1, file);
                fwrite(&a, sizeof(a), 1, file);
            }
            

            if (11 <= length) 
                printf("%d has edge case\n", i);
            else
                arr[length] += 1;

            for (uint8_t j = 0; j < length; j++)
            {
                for (uint8_t k = j + 1; k < length; k++)
                {
                    if (TABLE->buckets[i].point[j] == TABLE->buckets[i].point[k])
                    {
                        printf("%d has duplicate point\n", i);
                    }
                }
            }
        }

        for (int i = 0; i < 11; i++)
        {
            printf("%d \n", arr[i]);
        }

        fclose(file);

        hashmap_free(TABLE);

        free(TABLE);

        secp256k1_context_destroy(CTX);
    }


    int load_table()
    {
        FILE *file = fopen(TABLE_LOG, "rb");

        if (file == NULL) return printf("Error opening file");

        TABLE = (struct HashMap *)malloc(sizeof(struct HashMap));

        secp256k1_context *CTX = secp256k1_context_create(SECP256K1_CONTEXT_NONE);

        int good = secp256k1_load_discrete_log(CTX);

        if (!good) return printf("erro loading dlp");

        uint32_t current = 0;

        uint8_t length = 0;

        for (uint32_t i = 0; i < HASHMAP_SIZE; i++)
        {
            fread(&length, sizeof(length), 1, file);

            TABLE->buckets[i] = (current << 5) | length;

            for (uint8_t j = 0; j < length; j++)
            {
                fread(&(TABLE->nodes[current + j].point), sizeof(uint64_t), 1, file);
                fread(&(TABLE->nodes[current + j].head), sizeof(uint32_t), 1, file);
            }

            current = current + length;
        }

        fclose(file);

        return 0;
    }

}

void test()
{
    fill_random(INP, sizeof(INP));

    printf("inp: %s\n", hex_bytes(INP, 32).c_str());

    thread_encrypt();

    thread_decrypt();

    printf("inp: %s\n", hex_bytes(DEC, 32).c_str());

    free(TABLE);
}