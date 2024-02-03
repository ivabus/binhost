# BinHost

> HTTP server to easily serve (prebuilt) binaries for any (UNIX-like) platform with authenticity check

## Installation

```shell
cargo install --git https://github.com/ivabus/binhost
```

## Server usage

List options with `--help`

Make sure to use proxy with rate limiter in prod.

#### Directory structure

Directory, passed to `binhost` `--dir` option (defaults to `./bin`) should look like (for `hello` binary)

Note: list of binaries will be refreshed every 5 minutes (by default, see `--refresh` option)

```tree
bin
└── hello
    ├── Darwin
    │   ├── arm64
    │   │   └── hello
    │   └── x86_64
    │       └── hello
    └── Linux
        └── aarch64
            └── hello
```

#### Runners

Runner is a (necessary) subprogram, that checks ED25519 signature of a binary file and needs to be statically compiled for every platform, that could use binaries from `binhost` server. 

Directory, passed to `binhost` `--runners-dir` option (defaults to `./runners`) should look like (for `Linux-x86_64`, `Linux-aarch64` and `Darwin-arm64` compiled runners)

```tree
runners
├── runner-Darwin-arm64
├── runner-Linux-aarch64
└── runner-Linux-x86_64
```

## Client usage

### Execute specific binary <bin> with manifest validity check

Manifest validity check provides a fully-secured binary distribution chain.

```shell
curl ADDRESS:PORT/<bin> | KEY=... sh
```

`KEY` first few symbols from hex representation of SHA256 sum of manifest (printed to stdout on `binhost` startup).

Additional arguments are set with `ARGS` environment variable

Only this option should be considered as secure.

### Execute specific binary <bin> without validity check

```shell
curl ADDRESS:PORT/<bin> | sh
```

### Download and reuse script

```shell
curl ADDRESS:PORT/<bin> -o script.sh
./script.sh # Execute preloaded bin configuration
BIN=<newbin> ./script.sh # Execute newbin (download)
BIN=<newbin> EXTERNAL_ADDRESS=<newaddress> ./script.sh # Execute newbin from newaddress
```

### API

See full HTTP API in [API.md](./API.md)

## License

This project is licensed under [MIT License](./LICENSE)