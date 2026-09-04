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
        // Единый проход: reverse + markers($$) + xor, без промежуточных 4GB копий
        const std::string &src = m0;
        size_t n = src.size();
        size_t klen = m1.size();
        size_t full_groups = n / 5;
        size_t out_len = n + full_groups * 2;
        std::string middle;
        middle.reserve(out_len);
        size_t i = n;
        size_t ki = 0;
        while (i > 0) {
            size_t start = (i >= 5) ? (i - 5) : 0;
            size_t cnt = i - start;
            for (size_t r = cnt; r-- > 0;) {
                middle.push_back(src[start + r] ^ m1[ki]);
                if (++ki == klen) ki = 0;
            }
            i = start;
            if (cnt == 5) {
                middle.push_back('$' ^ m1[ki]);
                if (++ki == klen) ki = 0;
                middle.push_back('$' ^ m1[ki]);
                if (++ki == klen) ki = 0;
            }
        }
        std::string b = base64_encode(middle);
        return b;
    }
}

// generate repeated text of given byte size
std::string gen_data(size_t bytes) {
    std::string pattern =
        "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    std::string out;
    out.reserve(bytes);
    while (out.size() < bytes) {
        out += pattern.substr(0, std::min(bytes - out.size(), pattern.size()));
    }
    return out;
}

int main(int argc, char *argv[]) {
    if (argc < 4) {
        std::cerr << "Usage:\n"
                  << "  " << argv[0] << " <encode|decode> <key> <text> [iterations]\n"
                  << "  " << argv[0] << " --large <bytes> <key> [mode] [iterations]\n";
        return 1;
    }

    std::string mode;
    std::string key;
    std::string text;
    int iterations = 1000;

    bool large = (std::string(argv[1]) == "--large");
    if (large) {
        size_t bytes = std::atoll(argv[2]);
        key = argv[3];
        mode = (argc >= 5) ? argv[4] : "encode";
        iterations = (argc >= 6) ? std::atoi(argv[5]) : 3;
        text = gen_data(bytes);
    } else {
        mode = argv[1];
        key = argv[2];
        text = argv[3];
        iterations = (argc >= 5) ? std::atoi(argv[4]) : 1000;
    }
    bool isDecode = (mode == "decode");

    // Для больших decode: кодируем вход заранее (вне таймера), чтобы decode получил честный base64 вход
    std::string bench_input = text;
    if (large && isDecode) {
        bench_input = obxrac32b64(false, text, key);
    }
    size_t input_len = bench_input.size();

    int warmup = large ? 0 : std::max(iterations / 10, 100);
    for (int i = 0; i < warmup; ++i) {
        volatile auto w = obxrac32b64(isDecode, bench_input, key);
    }

    auto start = std::chrono::high_resolution_clock::now();
    std::string result;
    for (int i = 0; i < iterations; ++i) {
        result = obxrac32b64(isDecode, bench_input, key);
    }
    auto end = std::chrono::high_resolution_clock::now();

    double total_ms = std::chrono::duration<double, std::milli>(end - start).count();
    double avg_us = (total_ms * 1000.0) / iterations;
    double mbps = (double)input_len / (total_ms / 1000.0) / (1024.0 * 1024.0);

    std::cout << "C++ " << mode << " | input=" << input_len << "B"
              << " | iters=" << iterations
              << " | total=" << total_ms << "ms"
              << " | avg=" << avg_us << "us/iter"
              << " | " << mbps << " MB/s"
              << " | result_len=" << result.size() << "\n";

    return 0;
}
