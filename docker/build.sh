#!/usr/bin/env bash
# Build the reproducible cargo-near image.
#
#   docker/build.sh <cargo-near-version> <rust-version> [buildx output args...]
#
# Examples:
#   docker/build.sh 0.22.0 1.97.1 --output type=oci,dest=img.tar,rewrite-timestamp=true
#   docker/build.sh 0.22.0 1.97.1 --tag ghcr.io/near/cargo-near:0.22.0-rust-1.97.1 \
#     --output type=image,push=true,rewrite-timestamp=true --metadata-file meta.json
#
# The same inputs always produce the same image digest: SOURCE_DATE_EPOCH is
# derived from the Debian snapshot pinned in the Dockerfile, and the cargo-near
# tarball is checked against the sha256 published with its GitHub release.
set -euo pipefail

cargo_near_version=${1:?cargo-near version}
rust_version=${2:?rust version}
shift 2

dir=$(cd "$(dirname "$0")" && pwd)

snapshot=$(sed -n 's/^ARG DEBIAN_SNAPSHOT=//p' "$dir/Dockerfile")
source_date_epoch=$(date -u -d "${snapshot:0:8} ${snapshot:9:2}:${snapshot:11:2}:${snapshot:13:2}" +%s)

cargo_near_sha256=$(curl -fsSL "https://github.com/near/cargo-near/releases/download/cargo-near-v${cargo_near_version}/cargo-near-x86_64-unknown-linux-gnu.tar.gz.sha256" | cut -d' ' -f1)

exec docker buildx build \
  --platform linux/amd64 \
  --provenance=false \
  --sbom=false \
  --build-arg SOURCE_DATE_EPOCH="$source_date_epoch" \
  --build-arg RUST_VERSION="$rust_version" \
  --build-arg CARGO_NEAR_VERSION="$cargo_near_version" \
  --build-arg CARGO_NEAR_SHA256="$cargo_near_sha256" \
  "$@" \
  "$dir"
