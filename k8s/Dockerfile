######################
#### Running stage
######################
FROM ubuntu:24.04 as runtime

RUN apt update && apt install curl tzdata libpq5 libc6 -y

WORKDIR /srv
COPY ./target/release/app ./utxo-global-multi-sig-api
COPY ./config.toml ./
RUN chmod 550 utxo-global-multi-sig-api

ENV TZ=Asia/Ho_Chi_Minh \
    RUST_LOG=info

ENTRYPOINT [ "/srv/utxo-global-multi-sig-api" ]
