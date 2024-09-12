# utxo-global-multi-sig-api

## Migrate DB

Install Sqlx

```
cargo install sqlx-cli --no-default-features --features postgres
```

Add migrate

```
sqlx migrate add multi_sig_signers
```

Run Migrate

```
sqlx migrate run
```

## run test

```
cargo run --bin test_addrress
cargo run --bin test_multisign
cargo run --bin test_flow
```

## Check clippy

```
cargo clippy -- -D warnings
```
