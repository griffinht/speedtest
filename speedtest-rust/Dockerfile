FROM rust as build

COPY . .
RUN cargo build --release && ln -s $(cd ./target/release; pwd)/speedtest-rust /speedtest-rust

FROM gcr.io/distroless/cc

COPY --from=build /speedtest-rust /usr/local/bin/speedtest-rust

ENTRYPOINT ["speedtest-rust"]