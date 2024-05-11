FROM rust:latest

WORKDIR /nate

RUN cargo build --release

CMD ["./target/release/nate"]
