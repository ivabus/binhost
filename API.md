### Get list of binaries

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

### Get ed25519 signature

#### Request

```http request
GET /bin/<BIN>/<PLATFORM>/<ARCH>/sign HTTP/1.1
```

### Get manifest

Manifest is a file, that contains ED25519 public key and SHA256sums list

#### Request

```http request
GET /runner/manifest HTTP/1.1
```

### Get binary runner

#### Request

```http request
GET /runner/<runner> HTTP/1.1
```