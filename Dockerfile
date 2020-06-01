FROM rust:1.43.0 AS builder
RUN USER=root cargo new --bin productivity
WORKDIR /productivity
RUN curl -L https://github.com/golang-migrate/migrate/releases/download/v4.11.0/migrate.linux-amd64.tar.gz | tar xvz
RUN mv migrate.linux-amd64 migrate
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN echo 'fn main() {println!("Hello, world!");}' > src/main.rs
RUN echo "" > src/lib.rs
RUN cargo build --release
RUN rm src/*.rs
COPY src/ src/
COPY tests/ tests/
COPY db/ db/
COPY ./wait-for-it.sh ./wait-for-it.sh
RUN cargo clean
RUN cargo build --release

FROM rust:1.43.0 AS test
COPY --from=builder /productivity/migrate ./migrate
COPY . .
CMD ./wait-for-it.sh postgres:5432 -- migrate -database $POSTGRES_URL -path db/migrations up && cargo test -- --test-threads=1

FROM ubuntu:18.04 AS production
COPY --from=builder /productivity/db ./db
COPY --from=builder /productivity/wait-for-it.sh ./wait-for-it.sh
COPY --from=builder /productivity/target/release/productivity_bin ./productivity_bin
COPY --from=builder /productivity/migrate ./migrate
CMD ./wait-for-it.sh postgres:5432 -- ./migrate -database $POSTGRES_URL -path db/migrations up && RUST_BACKTRACE=full ./productivity_bin

