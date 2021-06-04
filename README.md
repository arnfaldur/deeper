# deeper

This is the repository for Arnaldur Bjarnason's and Jökull Máni Reynisson's final project for the spring semester of
2021. The project was completed with the tutelage of Hannes Högni Vilhjálmsson and examined by Torfi Ásgeirsson.

## Usage

To run this project you need to [install the rust toolchain](https://www.rust-lang.org/tools/install).

Download the code from this repository

```sh
git clone https://github.com/arnfaldur/deeper.git
```

compile and run the project

```sh
cd deeper
cargo build --release
cargo run --release
```

The `--release` flag is not strictly necessary, if it's removed the project is compiled in debug mode.