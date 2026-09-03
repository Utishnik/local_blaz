#include <algorithm>
#include <iostream>
#include <string>
#include <stdint.h>
#include <cstring>

std::string xor32(const std::string &sText, const std::string &sSecretKey) {
    if (sSecretKey.empty()) return sText;
    std::string result;
    result.reserve(sText.size());
    for (size_t chunk = 0; chunk < sText.size(); ++chunk) {
        result.push_back(sText[chunk] ^ sSecretKey[chunk % sSecretKey.size()]);
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

std::string rvbstr(std::string _key) {
    std::reverse(_key.begin(), _key.end());
    return _key;
}

std::string base64_decode(const std::string &input) {
    std::string out;
    out.reserve(input.size() * 3 / 4);
    int val = 0, valb = -8;
    for (unsigned char c : input) {
        if (c == '=') break;
        const char *p = strchr(BASE64_CHARS, c);
        if (!p) continue;
        val = (val << 6) + (p - BASE64_CHARS);
        valb += 6;
        if (valb >= 0) {
            out.push_back(char((val >> valb) & 0xFF));
            valb -= 8;
        }
    }
    return out;
}

std::string obxrac32b64(bool isDecode, std::string m0, std::string m1) {
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
        std::cerr << "Usage: " << argv[0] << " <encode|decode> <key> <text>\n";
        return 1;
    }
    std::string mode = argv[1];
    std::string key = argv[2];
    std::string text = argv[3];
    bool isDecode = (mode == "decode");
    std::cout << obxrac32b64(isDecode, text, key) << std::endl;
    return 0;
}
