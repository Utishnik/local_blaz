# obxrac32b64 (cross-lang and crossplatform algorithm obfuscation)
# C++ 
- Windows x64-arm64 (Open obxrac32b64.sinx to Visual Studio 2026) & build_mingw.bat
- Linux x64-aarch64 (build_linux_g++.sh || build_linux_clang++.sh)
- Mac OS (Open folder hash-obxrac32b64_build_macos -> Open README.md )
# Rust
```
cargo build --release

```
# js and node.js
* Open Folder js->obxrac32b64.js
* test obxrac32b64.html
# link to My Projects
|  C++
``` C++
#include "obxrac32b64.hpp"
``` 
|  js
``` C++
<script src="obxrac32b64.js"></script>
``` 
| python
``` python
import obxrac32b64_lib
```
# Example C++

``` C++
    #include <algorithm>
    #include <iostream>
    #include <string>
    #include <stdint.h>
    #include <cstring>
	#include "obxrac32b64.hpp"
	int main(){
	std::cout << "obxrac32b64 cryptor\n";
	std::cout << "e - encode\n";
    std::cout << "d - decode\n";
	bool dec = false;
        //	int k = _getch();
	std::cout <<">";
	uint8_t k = 0;
	std::cin >> k;
		dec = (k == 'e' ? false : true);
	std::string s_key = "";
	std::string s_data = "";
	std::cout <<" secret_key:";
	std::cin >> s_key;
	std::cin.ignore();
	std::cout <<" text:";
	std::getline(std::cin,s_data);
	std::cout << obxrac32b64(dec,s_data,s_key) << "\n";
	return 0;
	}
```
# Example Rust
``` Rust
fn main() -> Result<(), Box<dyn Error>> {
    println!("obxrac32b64 cryptor");
    let mode = prompt("mode (e - encode / d - decode)")?;
    let key = prompt("secret_key")?;
    let text = prompt("text")?;

    let start = Instant::now();
    if mode.starts_with('d') {
        match decode(&text, &key) {
            Ok(decrypted) => println!("Decrypted: {}", decrypted),
            Err(e) => eprintln!("Decode error: {}", e),
        }
    } else {
        println!("Encrypted: {}", encode(&text, &key));
    }
    println!("Elapsed: {:0.6}s", start.elapsed().as_secs_f64());

    Ok(())
}

```
# Example Python
``` python
import obxrac32b64_lib

def main():
    print("obxrac32b64 python build\n")
    print("e - encode\nd - decode\n")  
    cmd = input()
    dec = False if cmd == 'e' else True
    print("text:")
    text = input() 
    print("secret key:")
    key = input()
    result = obxrac32b64(dec, text, key)
    print("Result:", result)

if __name__ == "__main__":
    main()
```
# Example HTML+JS

``` js
obxrac32b64.encode(text,key);
obxrac32b64.decode(text,key);
```
