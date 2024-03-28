# talv

Chess bot written in Rust. Comes with a simple GUI.

## Commandline args

The GUI binary (`talv_ggez`) takes three commandline arguments. The first is a FEN string describing the position to start playing from. Remember to quote the string as valid FEN strings contain spaces and the binary only reads the first argument.
If no valid FEN string is provided the standard chess starting position is used.
The second and third describe who plays white and black respectively. If no known bot is put there, a human player is the default instead. The current list of bots are:

- `1` a first attempt at a minimax chess bot using a simple evaluation function.

## Build with cargo

To build it yourself install Rust and Cargo (use [rustup](https://rustup.rs/)). The pre-built binaries are of the `talv_ggez` client that has a GUI, the others are cumbersome to use. Clone the repo and run `cargo run --bin talv_ggez -- - - 1` to start playing against the bot in a GUI. For better performance compiling with `-r` (`--release`) will turn on optimisations.

## Download builds

- [Linux][linux]
- [macOS (Intel)][macos]
- [macOS (Apple Silicon)][macos-arm]
- [Windows][win]

[linux]: https://nightly.link/LFalch/talv/workflows/build/main/talv-x86_64-unknown-linux-musl.zip
[macos]: https://nightly.link/LFalch/talv/workflows/build/main/talv-x86_64-apple-darwin.zip
[macos-arm]: https://nightly.link/LFalch/talv/workflows/build/main/talv-aarch64-apple-darwin.zip
[win]: https://nightly.link/LFalch/talv/workflows/build/main/talv-x86_64-pc-windows-msvc.zip
