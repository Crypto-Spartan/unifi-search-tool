# Unifi Search Tool v2.0.1 - Download [Here](https://github.com/Crypto-Spartan/unifi-search-tool/releases/latest)
Does your unifi controller have lots of sites? Do you frequently have equipment returned from those sites and you can't remember where it's adopted in the controller? Enter Unifi Search Tool.

### How to Use

![examplev2](https://raw.githubusercontent.com/Crypto-Spartan/unifi-search-tool/master/screenshots/examplev2.png "examplev2")

1. Enter your username & password for your Unifi Controller

2. Enter your Unifi Controller domain/IP. You must include the proper http:// or https:// with the appropriate port number at the end, unless it runs on 80/443. (You will see this in the address bar of your browser when you open up your Unifi Controller.)

3. Enter the MAC Address of the device you're searching for

4. Click search

5. Profit

The tool will tell you which site in the controller that the device was adopted to. If it hasn't been adopted, the tool will tell you that the device could not be found.

## **Advanced**

### Pre-populate username, password, & URL fields

**\*\*\* This feature is not functional in the 2.0.0 release. It is planned to be added in the 2.1.0 release. If you require the pre-populated fields, you can download v1.4.1 [here](https://github.com/Crypto-Spartan/unifi-search-tool/releases/tag/1.4.1) \*\*\***

These instructions are for those that would like to add in their own defaults so that they don't need to re-enter their credentials or controller URL each time the program is opened. (This will only work for the installed version unless you decide to build the portable version from source.)

#### NOTE: If you choose to do this and credentials are stolen, I am not responsible. This is at your own risk.

1. Find `config.txt` within the install folder. (Default is C:\Program Files (x86)\Unifi Search Tool)

2. Add in your own values to the right of the `=` symbol

3. Save config.txt

Next time Unifi Search Tool is launched, it will have new pre-populated fields.

## Build From Source

![Minimum Rust: 1.65](https://img.shields.io/badge/Minimum%20Rust%20Version-1.65-brightgreen.svg)

1. Download the Zip of the source files and extract it

2. Open up a terminal in the directory

3. Run `cargo build --release` in the terminal

5. Go to the `target/release` directory to find the unifi-search-tool.exe file

### If you would like to optimize the binary for size

1. Install the appropriate toolchain and the rust-src component
```bash
$ rustup toolchain install nightly
$ rustup component add rust-src --toolchain nightly
```
2. Find your host's target triple
```bash 
$ rustc -vV
...
host: x86_64-unknown-linux-gnu
```
3. Run the build command
```bash
# Use that target triple when building with build-std.
# Add the =std,panic_abort to the option to make panic = "abort" Cargo.toml option work.
# See: https://github.com/rust-lang/wg-cargo-std-aware/issues/56
$ cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --release
```

- see https://github.com/johnthagen/min-sized-rust for more details
