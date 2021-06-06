# deeper

This is the repository for Arnaldur Bjarnason's and Jökull Máni Reynisson's final project for the spring semester of
2021. The project was completed with the tutelage of Hannes Högni Vilhjálmsson and examined by Torfi Ásgeirsson.

## Usage

To run this project you need to [install the rust toolchain](https://www.rust-lang.org/tools/install).

Download the code from this repository

```shell
git clone https://github.com/arnfaldur/deeper.git
```

compile and run the project:

```shell
cd deeper
cargo run --release
```

Note that compilation takes around five minutes.

To generate a new map, close the program and run:

```shell
cargo test --release
cargo run --release
```

The generation can take around 30 seconds.

The `--release` flag is not strictly necessary, if it's removed the project is compiled in debug mode. You should either
always use the flag or never.

