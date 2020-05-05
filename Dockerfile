FROM rust:1.42.0 AS base
EXPOSE 5555
RUN apt-get update \
    && apt-get install curl lsb-release -y \
    && curl -s https://packagecloud.io/install/repositories/golang-migrate/migrate/script.deb.sh | bash \
    && apt-get update \
    && apt-get install migrate=4.9.1 -y
WORKDIR /usr/src/productivity_back
COPY . .

FROM base as development
CMD ./wait-for-it.sh postgres:5432 -- migrate -database $POSTGRES_URL -path db/migrations up

FROM base as test
CMD ./wait-for-it.sh postgres:5432 -- migrate -database $POSTGRES_URL -path db/migrations up && cargo test -- --test-threads=1

FROM base as production
RUN cargo build --release
CMD ./wait-for-it.sh postgres:5432 -- migrate -database $POSTGRES_URL -path db/migrations up && ./target/release/productivity_bin

