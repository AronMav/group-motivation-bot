FROM rust:latest as rust-build
RUN apt update && apt -y install lld musl-tools
COPY src/ ./src/
COPY Cargo.toml ./
COPY log.yml ./
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo test --workspace --target x86_64-unknown-linux-musl && \
    cargo build --workspace --target x86_64-unknown-linux-musl --release

FROM gcr.io/distroless/static-debian12:latest
ENV BOT_ID=0000000
ENV BOT_NAME=Bot_name
ENV DB_PATH=/usr/local/bin/db.sqlite3
ENV KEY_WORD=спасибо
ENV LOG_PATH=/usr/local/bin/log.yml
ENV TELOXIDE_TOKEN=0000000
#COPY --from=rust-build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=rust-build /log.yml /usr/local/bin/log.yml
COPY --from=rust-build /target/x86_64-unknown-linux-musl/release/group-motivation-bot /usr/local/bin/group-motivation-bot
ENTRYPOINT ["/usr/local/bin/group-motivation-bot"]
CMD ["group-motivation-bot"]