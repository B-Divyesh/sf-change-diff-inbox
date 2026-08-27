FROM node:22-alpine AS web
WORKDIR /build
COPY package.json package-lock.json ./
COPY frontend/package.json frontend/package.json
RUN npm ci
COPY frontend frontend
RUN npm --workspace frontend run build

FROM rust:1.89-alpine AS backend
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY migrations migrations
COPY src src
ARG BUILD_SHA
# The deployer must attest the immutable source revision used for this image.
RUN test -n "$BUILD_SHA" && printf '%s' "$BUILD_SHA" | grep -Eq '^[0-9a-f]{40}$'
ENV BUILD_SHA=$BUILD_SHA
RUN cargo build --release --locked

FROM alpine:3.22
RUN apk add --no-cache ca-certificates libgcc && addgroup -S app && adduser -S -G app app
WORKDIR /app
COPY --from=backend /build/target/release/change-diff-inbox /usr/local/bin/change-diff-inbox
COPY --from=web /build/frontend/dist /app/frontend
RUN mkdir -p /app/data && chown -R app:app /app
USER app
ENV PORT=8080 \
    FRONTEND_DIR=/app/frontend \
    DATABASE_URL=sqlite:///app/data/change-diff.db?mode=rwc \
    RUST_LOG=change_diff_inbox=info,tower_http=info
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD wget -q -O /dev/null http://127.0.0.1:8080/health || exit 1
CMD ["change-diff-inbox"]
