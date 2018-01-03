FROM rust:1.22

COPY . /usr/src/app
WORKDIR /usr/src/app

RUN cargo build --release

CMD ./target/release/visit
