FROM node AS frontend
COPY website .
RUN  npm install --registry https://registry.npmmirror.com && npm run build

FROM --platform=linux/amd64 messense/rust-musl-cross:x86_64-musl AS amd64
COPY . .
RUN cd pikpak_web && cargo install --path . --root /


FROM --platform=linux/amd64 messense/rust-musl-cross:aarch64-musl AS arm64
COPY . .
RUN cargo install --path . --root /

FROM ${TARGETARCH} AS builder

FROM scratch
COPY --from=builder /bin/pikpak_web /pikpak_web
COPY --from=frontend /dist /dist
STOPSIGNAL SIGINT
ENTRYPOINT ["/pikpak_web"]