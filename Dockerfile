FROM rust:slim-stretch as build
WORKDIR /service

RUN USER=root cargo new --bin enokey
WORKDIR /service/enokey

RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

RUN rustup install nightly-2019-07-09

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo +nightly-2019-07-09 build

COPY ./Rocket.toml ./Rocket.toml
COPY ./src ./src

RUN touch src/main.rs
RUN cargo +nightly-2019-07-09 build

FROM debian:stretch-slim
WORKDIR /enokey
RUN mkdir keyfiles
RUN mkdir data

RUN apt-get update \
    && apt-get install -y libssl1.1 ca-certificates openssh-client --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /service/enokey/target/debug/enokey .
COPY ./static ./static
COPY ./templates ./templates
COPY ./Rocket.toml ./Rocket.toml

ENV ROCKET_ENV production

RUN adduser --disabled-password --gecos '' enokey
RUN mkdir /home/enokey/.ssh
RUN chown -R enokey /home/enokey/
RUN chown -R enokey .
COPY /docker-entrypoint.sh /
RUN chmod o+x /docker-entrypoint.sh
ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["./enokey"]
