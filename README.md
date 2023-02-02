# algorand-vanity
Generate Vanity addresses on Algorand

## Running
```
export RUSTFLAGS='-C target-cpu=native'
cargo run --release -- --cpu PREFIX
```

Replace `PREFIX` with desired prefix.
It must consist of characters available in addresses only (A-Z, 2-7).
Providing illegal characters will result in an infinite loop.
