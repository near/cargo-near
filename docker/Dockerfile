# syntax=docker/dockerfile:1.10
#
# Reproducible build image for `cargo near build reproducible-wasm`.
# Published as `ghcr.io/near/cargo-near:<cargo-near>-rust-<rust>`.
#
# Every input is pinned (base image digest, apt snapshot, rustup-init and
# cargo-near checksums), so building this file twice with the same args and
# SOURCE_DATE_EPOCH yields the same image digest.

ARG DEBIAN_IMAGE=debian:trixie-20260918-slim@sha256:a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a

FROM ${DEBIAN_IMAGE} AS fetch
ARG RUSTUP_VERSION=1.29.1
ARG RUSTUP_SHA256=dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71
ARG CARGO_NEAR_VERSION
ARG CARGO_NEAR_SHA256
ADD --checksum=sha256:${RUSTUP_SHA256} \
    https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/x86_64-unknown-linux-gnu/rustup-init \
    /fetch/rustup-init
ADD --checksum=sha256:${CARGO_NEAR_SHA256} \
    https://github.com/near/cargo-near/releases/download/cargo-near-v${CARGO_NEAR_VERSION}/cargo-near-x86_64-unknown-linux-gnu.tar.gz \
    /fetch/cargo-near.tar.gz
RUN tar -xzf /fetch/cargo-near.tar.gz -C /fetch --strip-components=1 \
    && chmod 755 /fetch/rustup-init /fetch/cargo-near

FROM ${DEBIAN_IMAGE}
ARG DEBIAN_SNAPSHOT=20260918T000000Z
# useradd writes this into /etc/shadow, so it must be fixed for reproducibility.
ARG SOURCE_DATE_EPOCH

# System packages from a fixed snapshot.debian.org date. Same package set as
# `sourcescan/cargo-near`, so images are drop-in replacements.
RUN rm -f /etc/apt/sources.list.d/debian.sources \
    && printf '%s\n' \
       "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${DEBIAN_SNAPSHOT} trixie main" \
       "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${DEBIAN_SNAPSHOT} trixie-updates main" \
       "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/${DEBIAN_SNAPSHOT} trixie-security main" \
       > /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
       curl \
       build-essential \
       ca-certificates \
       clang \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/* /var/log/* /var/cache/ldconfig/aux-cache \
    && groupadd -g 1000 near \
    && useradd -m -d /home/near -s /bin/bash -g near -u 1000 near \
    && rm -f /var/log/lastlog /var/log/faillog

USER near

ARG RUST_VERSION
ENV HOME=/home/near \
    RUSTUP_TOOLCHAIN=${RUST_VERSION} \
    CARGO_HOME=/home/near/.cargo \
    RUSTUP_HOME=/home/near/.rustup \
    PATH=/home/near/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

# `near-verify-rs` runs the container as the host uid, so the home directory
# stays world-writable, as in `sourcescan/cargo-near`.
RUN --mount=type=bind,from=fetch,source=/fetch,target=/fetch \
    /fetch/rustup-init -y --no-modify-path --profile minimal \
       --default-toolchain ${RUST_VERSION} --target wasm32-unknown-unknown \
    && install -m 755 /fetch/cargo-near ${CARGO_HOME}/bin/cargo-near \
    && rm -rf ${RUSTUP_HOME}/tmp/* ${RUSTUP_HOME}/downloads/* \
    && chmod -R a+rwx ${HOME}
