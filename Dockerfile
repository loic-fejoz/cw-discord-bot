FROM rust:1.59 as chef
RUN cargo install cargo-chef --locked

FROM chef as planner
WORKDIR /cw-bot
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as cacher
RUN USER=root apt-get update && apt-get install -y libasound2-dev
WORKDIR /cw-bot
COPY --from=planner /cw-bot/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM chef as builder
RUN USER=root apt-get update && apt-get install -y libasound2-dev
WORKDIR /cw-bot
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /cw-bot/target target
RUN cargo build --release --bin cw-bot

FROM rust:1.59 as runtime
RUN apt-get update && apt-get install -y ffmpeg libasound2 libmp3lame0
WORKDIR /cw-bot
COPY --from=builder /cw-bot/target/release/cw-bot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/cw-bot"]

#RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu --bin cw-bot
