FROM rust:slim-stretch as build
WORKDIR /service

RUN USER=root cargo new --bin enokey
WORKDIR /service/enokey

RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config --no-install-recommends

RUN rustup install nightly

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo +nightly build

COPY ./Rocket.toml ./Rocket.toml
COPY ./src ./src

RUN touch src/main.rs
RUN cargo +nightly build

FROM debian:stretch-slim
WORKDIR /enokey
RUN mkdir keyfiles

RUN apt-get update \
    && apt-get install -y libssl1.1 ca-certificates

COPY --from=build /service/enokey/target/debug/enokey .
COPY ./static ./static
COPY ./Rocket.toml ./Rocket.toml

ENV ROCKET_ENV production
ENV ROCKET_TEMPLATE_DIR static

ENTRYPOINT "./enokey"