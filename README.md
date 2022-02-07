# wrdl
My take on a wordle solver

## Compiling

Some extra steps are necessary here because this project depends on the `edition2021` Cargo feature.

1. Install the `rustup` package using your favorite package manager
2. Run `$ rustup toolchain install nightly` to make the nightly channel toolchain available
3. To select that toolchain, inside the project root directory, run `$ rustup override set nightly`
4. Finally, run `$ cargo build --release`
5. Done! The `wrdl` executable can be found in the `target/release` directory

## Running

`wrdl <LANGUAGE>`, where `LANGUAGE` is one of `en`/`pt`. The program will guide you through the rest!
