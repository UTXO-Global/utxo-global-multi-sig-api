######################
#### Running stage
######################
FROM ubuntu:22.04 as runtime

RUN apt update && apt install curl tzdata libpq5 -y
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    deployer

WORKDIR /srv
COPY ./target/release/utxo-global-multi-sig-api ./
COPY ./config.toml ./
RUN chmod 550 utxo-global-multi-sig-api

USER deployer:deployer

ENV TZ=Asia/Ho_Chi_Minh \
    RUST_LOG=info

USER deployer

ENTRYPOINT [ "/srv/utxo-global-multi-sig-api" ]
