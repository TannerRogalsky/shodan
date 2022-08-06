FROM clux/muslrust:stable as build
ADD ./ ./
RUN cargo build --release -v

FROM gcr.io/distroless/static:nonroot
COPY --chown=nonroot:nonroot --from=build /volume/target/x86_64-unknown-linux-musl/release/spookybot /app/
EXPOSE 8080
ENTRYPOINT ["/app/spookybot"]