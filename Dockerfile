# ------------------------------------------------------------------------------
# Base Runtime Stage
# ------------------------------------------------------------------------------

FROM debian:stable-slim AS runtime

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y sqlite3 libsqlite3-dev libssl-dev ca-certificates

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.39 as cargo-build

WORKDIR /usr/src/

# RUN USER=root cargo new ingredients_bot
# WORKDIR /usr/src/ingredients_bot
# RUN touch src/lib.rs
# RUN mv src/main.rs src/server.rs
# COPY Cargo.toml ./
# RUN cargo build --release --bin ingredients_bot_server

WORKDIR /usr/src/ingredients_bot
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

# RUN rm -rf target/release/deps/ingredients_bot*
# RUN rm -rf target/release/ingredients_bot*

COPY src src
RUN cargo build --release
RUN cargo install --bin ingredients_bot_server --path . --verbose

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM runtime

COPY food.db /
ENV FOOD_DB=/food.db

COPY --from=cargo-build /usr/local/cargo/bin/ingredients_bot_server /bin

USER 1000

CMD ["ingredients_bot_server"]
