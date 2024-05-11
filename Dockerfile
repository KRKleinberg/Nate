FROM rust:latest

WORKDIR /nate

COPY . .

RUN cargo build --release

CMD ["./target/release/nate"]
