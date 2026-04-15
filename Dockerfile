FROM rust:1.89-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

COPY . .

RUN WASM_BINDGEN_VERSION="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/^version = "|"/, "", $0); print; exit }' Cargo.lock)" \
    && cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION"

RUN ./scripts/build-web.sh release

FROM caddy:2.11-alpine

COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/web /srv

EXPOSE 80 443
