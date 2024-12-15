################################################################################
# Create a stage for building the application.
FROM --platform=$BUILDPLATFORM rust:1.83-alpine AS build
WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git zig && \
    cargo install --locked cargo-zigbuild && \
    rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Build the application.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=restic-crd,target=restic-crd \
    --mount=type=bind,source=.cargo,target=.cargo \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo zigbuild --locked --release --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl && \
    mkdir /app/linux && \
    cp target/aarch64-unknown-linux-musl/release/restic-operator /app/linux/arm64 && \
    cp target/x86_64-unknown-linux-musl/release/restic-operator /app/linux/amd64

################################################################################
FROM alpine:3.18 AS final
ARG TARGETPLATFORM

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /app/${TARGETPLATFORM} /bin/restic-operator

# What the container should run when it is started.
ENTRYPOINT ["/bin/restic-operator"]
