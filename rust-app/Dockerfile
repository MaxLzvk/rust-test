FROM docker.io/library/rust:1.88-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml .
COPY src/ src/
RUN cargo build --release

FROM docker.io/library/alpine:3.19
WORKDIR /app
COPY --from=build /app/target/release/maze-runner .
COPY static/ static/
RUN mkdir -p data
EXPOSE 8080
CMD ["./maze-runner"]
