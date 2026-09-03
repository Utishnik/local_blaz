#include <algorithm>
#include <iostream>
#include <string>
#include <stdint.h>
#include <cstring>
#include <chrono>
#include <cstdlib>

std::string xor32(const std::string &sText, const std::string &sSecretKey) {
    std::string result(sText.size(), '\0');
    size_t klen = sSecretKey.size();
    if (klen == 0) return result;
    size_t ki = 0;
    for (size_t chunk = 0; chunk < sText.size(); ++chunk) {
        result[chunk] = sText[chunk] ^ sSecretKey[ki];
        if (++ki == klen) ki = 0;
    }
    return result;
}

std::string addc(int s, const std::string &i) {
    std::string od;
    od.reserve(i.size() + (i.size() / s) * 2);
    int d = 0;
    for (char c : i) {
        od += c;
        if (++d == s) {
            od += "$$";
            d = 0;
        }
    }
    return od;
}

std::string addce(const std::string &s, int group_size = 5) {
    std::string r;
    r.reserve(s.size());
    size_t i = 0;
    while (i < s.size()) {
        size_t take = std::min((size_t)group_size, s.size() - i);
        r.append(s, i, take);
        i += take + 2;
    }
    return r;
}

static const char *BASE64_CHARS =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// lookup-таблица для decode: индекс символа -> значение (0x100 если невалидный)
static int b64_lookup[256];
static bool b64_init = [] {
    for (int i = 0; i < 256; ++i) b64_lookup[i] = 0x100;
    for (int i = 0; i < 64; ++i) b64_lookup[(unsigned char)BASE64_CHARS[i]] = i;
    return true;
}();

std::string base64_encode(const std::string &input) {
    std::string out;
    out.reserve(((input.size() + 2) / 3) * 4);
    int val = 0, valb = -6;
    for (unsigned char c : input) {
        val = (val << 8) + c;
        valb += 8;
        while (valb >= 0) {
            out.push_back(BASE64_CHARS[(val >> valb) & 0x3F]);
            valb -= 6;
        }
    }
    if (valb > -6) out.push_back(BASE64_CHARS[((val << 8) >> (valb + 8)) & 0x3F]);
    while (out.size() % 4) out.push_back('=');
    return out;
}

std::string rvbstr(const std::string &_key) {
    std::string out = _key;
    std::reverse(out.begin(), out.end());
    return out;
}

std::string base64_decode(const std::string &input) {
    std::string out;
    out.reserve(input.size() * 3 / 4);
    int val = 0, valb = -8;
    for (unsigned char c : input) {
        if (c == '=') break;
        int v = b64_lookup[c];
        if (v == 0x100) continue;
        val = (val << 6) + v;
        valb += 6;
        if (valb >= 0) {
            out.push_back(char((val >> valb) & 0xFF));
            valb -= 8;
        }
    }
    return out;
}

std::string obxrac32b64(bool isDecode, const std::string &m0, const std::string &m1) {
    if (isDecode) {
        std::string a = base64_decode(m0);
        std::string b = xor32(a, m1);
        b = addce(b);
        b = rvbstr(b);
        return b;
    } else {
        std::string a = rvbstr(m0);
        a = addc(5, a);
        std::string b = xor32(a, m1);
        b = base64_encode(b);
        return b;
    }
}

int main(int argc, char *argv[]) {
    if (argc < 4) {
        std::cerr << "Usage: " << argv[0] << " <encode|decode> <key> <text> [iterations]\n";
        return 1;
    }

    std::string mode = argv[1];
    std::string key = argv[2];
    std::string text = argv[3];
    int iterations = (argc >= 5) ? std::atoi(argv[4]) : 1000;
    bool isDecode = (mode == "decode");

    int warmup = std::max(iterations / 10, 100);
    for (int i = 0; i < warmup; ++i) {
        volatile auto w = obxrac32b64(isDecode, text, key);
    }

    auto start = std::chrono::high_resolution_clock::now();
    std::string result;
    for (int i = 0; i < iterations; ++i) {
        result = obxrac32b64(isDecode, text, key);
    }
    auto end = std::chrono::high_resolution_clock::now();

    double total_ms = std::chrono::duration<double, std::milli>(end - start).count();
    double avg_us = (total_ms * 1000.0) / iterations;

    std::cout << "C++ " << mode << " | iters=" << iterations
              << " | total=" << total_ms << "ms"
              << " | avg=" << avg_us << "us/iter"
              << " | result_len=" << result.size() << "\n";

    return 0;
}
