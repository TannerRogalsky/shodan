FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM clux/muslrust:stable as build
RUN cargo install cargo-chef
COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
ADD ./ ./
RUN cargo build --release -v

FROM gcr.io/distroless/static:nonroot
COPY --chown=nonroot:nonroot --from=build /volume/target/x86_64-unknown-linux-musl/release/spookybot /app/
EXPOSE 8080
ENTRYPOINT ["/app/spookybot"]