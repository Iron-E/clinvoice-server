ARG RUST_VERSION=1.76.0

################################################################################
# xx is a helper for cross-compilation.
# See https://github.com/tonistiigi/xx/ for more information.
FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.3.0 AS xx

################################################################################
# Create a stage for building the application.
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app

# Copy cross compilation utilities from the xx stage.
COPY --from=xx / /

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git file

# This is the architecture you’re building for, which is passed in by the builder.
# Placing it here allows the previous steps to be cached across architectures.
ARG TARGETPLATFORM

# Install cross compilation build dependencies.
RUN xx-apk add --no-cache musl-dev gcc openssl openssl-dev

ENV OPENSSL_DIR=/usr

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies, a cache mount to /usr/local/cargo/git/db
# for git repository dependencies, and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=sqlx-data.json,target=sqlx-data.json \
    --mount=type=cache,target=/app/target/,id=rust-cache-winvoice-server-${TARGETPLATFORM} \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    xx-cargo build --locked --release --target-dir ./target && \
    cp ./target/$(xx-cargo --print-target-triple)/release/winvoice-server /bin/winvoice-server && \
    xx-verify /bin/winvoice-server

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
#
# The example below uses the alpine image as the foundation for running the app.
# By specifying the "3.18" tag, it will use version 3.18 of alpine. If
# reproducability is important, consider using a digest
# (e.g., alpine@sha256:664888ac9cfd28068e062c991ebcff4b4c7307dc8dd4df9e728bedde5c449d91).
FROM alpine:3.18 AS final

LABEL org.opencontainers.image.authors="Iron-E <code.iron.e@gmail.com>"
LABEL org.opencontainers.image.description="winvoice-server docker image"
LABEL org.opencontainers.image.documentation="https://github.com/Iron-E/winvoice-server"
LABEL org.opencontainers.image.license="GPL-3.0-only"
LABEL org.opencontainers.image.source="https://github.com/Iron-E/winvoice-server"
LABEL org.opencontainers.image.title="winvoice-server"
LABEL org.opencontainers.image.url="https://github.com/Iron-E/winvoice-server"
LABEL org.opencontainers.image.vendor="Iron-E <code.iron.e@gmail.com>"
LABEL org.opencontainers.image.version="0.1.0"

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG GID=10001
ARG UID=$GID

RUN addgroup --system --gid "${GID}" winvoice && \
    adduser --system --uid "${UID}" server-runner

USER server-runner

# Copy the executable from the "build" stage.
COPY --from=build /bin/winvoice-server /bin/

# make the state dir (for logs)
RUN mkdir -p ~/.local/state

# What the container should run when it is started.
ENTRYPOINT ["/bin/winvoice-server"]
