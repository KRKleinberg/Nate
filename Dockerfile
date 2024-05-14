FROM rust:latest AS chef
RUN cargo install cargo-chef
RUN apt-get update && \
	apt-get -y install \
	libprotobuf-dev \
	protobuf-compiler \
	cmake \
	libopus-dev \
	build-essential \
	autoconf \
	automake \
	libtool \
	m4 \
	yt-dlp \
	ffmpeg
WORKDIR /nate

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /nate/recipe.json recipe.json
RUN RUST_BACKTRACE=full cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN RUST_BACKTRACE=1 cargo build --release --bin nate

FROM debian:bookworm-slim AS runtime
RUN cmake --version
WORKDIR /nate
COPY --from=builder /nate/target/release/nate /usr/local/bin
ENTRYPOINT [ "/usr/local/bin/nate" ]
