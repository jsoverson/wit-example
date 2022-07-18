# Install wit-bindgen

```sh
$ cargo install --git https://github.com/bytecodealliance/wit-bindgen wit-bindgen-cli
```

## Generate Host exports

```sh
wit-bindgen wasmtime --out-dir host --export wapc-host.wit --import wapc-guest.wit
```

## Generate Guest imports

```sh
wit-bindgen rust-wasm --out-dir guest --export wapc-guest.wit --import wapc-host.wit
```
