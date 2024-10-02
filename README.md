# VibraDB
## What is Vibra?
Vibra is a powerful, real-time key-value store that is thread-safe. Vibra takes inspiration from Laravel's Eloquent and SQLite.

Along with its ease-of-use and real-time capabilities, Vibra is powerfully encrypted using [LAQ-Fort](https://github.com/zanderlewis/laq-fort) with a fractal depth of 250 and a AES multiplier of 250 (by default), meaning you have 500 layers of standard encryption. LAQ-Fort has built-in triple Kyber encryption, along with the custom depth fractal encryption and custom multiplier of AES, ensuring your data is extra secure while keeping the speed of encryption and decryption.

## Installation
Vibra can be added to your `Cargo.toml` file like so:
```toml
[package]
name = "my_vibra_project"
version = "0.0.1"
edition = "2021"

[dependencies]
vibradb = <vibra_version_here>
```

## Usage
null
