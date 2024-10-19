FROM node:18.20.4-slim AS frontend
WORKDIR /app
COPY website/package.json website/package-lock.json* ./
RUN npm install --registry https://registry.npmmirror.com
COPY website .
RUN npm run build

FROM messense/rust-musl-cross:x86_64-musl AS amd64
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY pikpak_core/Cargo.toml ./pikpak_core/
COPY pikpak_web/Cargo.toml ./pikpak_web/
RUN mkdir -p pikpak_core/src pikpak_web/src && \
    echo "fn main() {}" > pikpak_core/src/lib.rs && \
    echo "fn main() {}" > pikpak_web/src/main.rs
RUN cargo build --release
COPY . .
RUN cargo install --path ./pikpak_web --root /

FROM messense/rust-musl-cross:aarch64-musl AS arm64
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY pikpak_core/Cargo.toml ./pikpak_core/
COPY pikpak_web/Cargo.toml ./pikpak_web/
RUN mkdir -p pikpak_core/src pikpak_web/src && \
    echo "fn main() {}" > pikpak_core/src/lib.rs && \
    echo "fn main() {}" > pikpak_web/src/main.rs
RUN cargo build --release
COPY . .
RUN cargo install --path ./pikpak_web --root /


FROM ${TARGETARCH} AS builder

FROM scratch
COPY --from=builder /bin/pikpak_web /pikpak_web
COPY --from=frontend /app/dist /dist
STOPSIGNAL SIGINT
ENTRYPOINT ["/pikpak_web"]