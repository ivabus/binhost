# BinHost

> HTTP server to easily serve (prebuilt) binaries for any (UNIX-like) platform

## Installation

```shell
cargo install --git https://github.com/ivabus/binhost
```

## Server usage

List options with `--help`

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

## Client usage

### Get available binaries

#### Request

```http request
GET / HTTP/1.1
```

#### Example response

```
- hello (platforms: ["Linux-aarch64", "Darwin-arm64", "Darwin-x86_64"])
```

### Get script for specific binary (suitable for `curl | sh` syntax)

This script will determine platform and arch and download necessary binary (and check hashsum if `sha256sum` binary is present in `$PATH`).

#### Request

```http request
GET /<BIN> HTTP/1.1
```

### Get binary for specific platform

#### Request

```http request
GET /bin/<BIN>/<PLATFORM>/<ARCH> HTTP/1.1
```

### Get sha256 hash of binary for specific platform

#### Request

```http request
GET /bin/<BIN>/<PLATFORM>/<ARCH>/sha256 HTTP/1.1
```

#### Example response

```text
a5d1fba1c28b60038fb1008a3c482b4119070a537af86a05046dedbe8f85e18d  hello
```

## License

This project is licensed under [MIT License](./LICENSE)