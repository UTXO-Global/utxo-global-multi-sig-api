# utxo-global-multi-sig-api

## run test

cargo run --bin test_multisign

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
