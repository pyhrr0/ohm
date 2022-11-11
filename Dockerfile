FROM rust:1.65.0 as builder
WORKDIR /usr/src/ohm-wallet
COPY . .
RUN rustup component add rustfmt && \
    cargo install --path .

FROM debian:buster-slim
RUN apt update && \
    apt install libsqlite3.0

COPY --from=builder /usr/src/ohm-wallet/ohm.cfg_example /etc/ohm/config.yml
COPY --from=builder /usr/local/cargo/bin/ohm-server /usr/local/bin/ohm-server
COPY --from=builder /usr/local/cargo/bin/ohm-client /usr/local/bin/ohm-client

RUN useradd -m ohm
USER ohm
CMD ["ohm-client"]
