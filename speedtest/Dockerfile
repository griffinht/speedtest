FROM rust as build

COPY . .
RUN cargo build --release && ln -s $(cd ./target/release; pwd)/speedtest /speedtest

FROM gcr.io/distroless/cc

COPY --from=build /speedtest /usr/local/bin/speedtest

ENTRYPOINT ["speedtest"]