FROM rust:latest AS chef
RUN cargo install cargo-chef
WORKDIR /nate

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /nate/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin nate

FROM ubuntu:latest AS runtime
WORKDIR /nate
COPY --from=builder /nate/target/release/nate /usr/local/bin
ENTRYPOINT [ "/usr/local/bin/nate" ]
