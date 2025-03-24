FROM lukemathwalker/cargo-chef:latest AS chef
RUN cargo install sqlx-cli
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
ENV DATABASE_URL=sqlite:./database.db
RUN sqlx database create
RUN sqlx migrate run
RUN cargo build --release
RUN mv ./target/release/mst-bot ./app

FROM debian:stable-slim AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]
