FROM debian:stable-slim AS runtime

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y sqlite3 libsqlite3-dev

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.39 as cargo-build

WORKDIR /usr/src/

RUN USER=root cargo new ingredients_bot
WORKDIR /usr/src/ingredients_bot
COPY Cargo.toml ./
RUN cargo build --release

RUN rm -rf target/release/deps/ingredients_bot*

COPY src src
RUN cargo install --path .

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM runtime

COPY food.db /
ENV FOOD_DB=/food.db

COPY --from=cargo-build /usr/local/cargo/bin/ingredients_bot /bin

USER 1000

CMD ["ingredients_bot"]
