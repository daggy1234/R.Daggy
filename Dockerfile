FROM lukemathwalker/cargo-chef as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM lukemathwalker/cargo-chef as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM rust as runtime
WORKDIR /app
COPY --from=builder /app/target/release/rdaggy /usr/local/bin
ENTRYPOINT ["/usr/local/bin/rdaggy"]