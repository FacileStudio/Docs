FROM oven/bun:1 AS client-build
WORKDIR /client
COPY apps/client/package.json apps/client/bun.lock* ./
RUN bun install --frozen-lockfile
COPY apps/client/ .
RUN bun run build

FROM rust:1-bookworm AS api-build
WORKDIR /repo/apps/api
COPY apps/api/Cargo.toml apps/api/Cargo.lock apps/api/rust-toolchain.toml apps/api/build.rs ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo '' > src/lib.rs && mkdir migrations \
    && cargo build --release --locked && rm -rf src migrations
COPY apps/api ./
COPY apps/client/package.json /version/package.json
COPY scripts/version.sh /version/version.sh
RUN touch src/main.rs src/lib.rs && APP_VERSION="$(sh /version/version.sh /version/package.json)" && export APP_VERSION \
    && cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
COPY --from=api-build /repo/apps/api/target/release/api /api
COPY --from=client-build /client/build /client
ENV CLIENT_DIR=/client
ENV PORT=4000
ENV DATA_DIR=/data
EXPOSE 4000
HEALTHCHECK --interval=10s --timeout=5s --retries=3 CMD ["/api", "healthcheck"]
USER nonroot:nonroot
ENTRYPOINT ["/api"]
