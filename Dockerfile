FROM ubuntu:24.04 as builder

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libopus-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y

WORKDIR /build

COPY rust-toolchain.toml .
COPY Cargo.toml .
COPY Cargo.lock .
COPY src/ src/
COPY application/ application/
COPY common/ common/
COPY domain/ domain/
COPY infrastructure/ infrastructure/
COPY lib/ lib/

RUN $HOME/.cargo/bin/cargo build --release

FROM ubuntu:24.04

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y ca-certificates
RUN apt-get clean

WORKDIR /app

COPY --from=builder /build/target/release/discord-bot app
COPY resource/ resource/

ENTRYPOINT ["./app"]
