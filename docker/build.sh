#!/usr/bin/env bash
# Build the reproducible cargo-near image.
#
#   docker/build.sh <cargo-near-version> <rust-version> [buildx output args...]
#
# Examples:
#   docker/build.sh 0.22.0 1.97.1 --output type=oci,dest=img.tar,oci-mediatypes=true,rewrite-timestamp=true
#   docker/build.sh 0.22.0 1.97.1 --tag ghcr.io/near/cargo-near:0.22.0-rust-1.97.1 \
#     --output type=image,push=true,oci-mediatypes=true,rewrite-timestamp=true --metadata-file meta.json
#
# BuildKit is pinned below. Use matching export settings, as above, to compare
# local and published digests. Compression uses this pinned BuildKit's defaults.
set -euo pipefail

cargo_near_version=${1:?cargo-near version}
rust_version=${2:?rust version}
shift 2

dir=$(cd "$(dirname "$0")" && pwd)

snapshot=$(sed -n 's/^ARG DEBIAN_SNAPSHOT=//p' "$dir/Dockerfile")
snapshot_date="${snapshot:0:8} ${snapshot:9:2}:${snapshot:11:2}:${snapshot:13:2}"
# Try GNU date, then BSD date (macOS).
source_date_epoch=$(date -u -d "$snapshot_date" +%s 2>/dev/null \
  || date -u -j -f '%Y%m%d %H:%M:%S' "$snapshot_date" +%s)

cargo_near_sha256=$(curl -fsSL "https://github.com/near/cargo-near/releases/download/cargo-near-v${cargo_near_version}/cargo-near-x86_64-unknown-linux-gnu.tar.gz.sha256" | cut -d' ' -f1)

builder=$(docker buildx create --driver docker-container \
  --driver-opt image=moby/buildkit:v0.30.0@sha256:0168606be2315b7c807a03b3d8aa79beefdb31c98740cebdffdfeebf31190c9f)
trap 'docker buildx rm "$builder" >/dev/null' EXIT

docker buildx build \
  --builder "$builder" \
  --platform linux/amd64 \
  --provenance=false \
  --sbom=false \
  --build-arg SOURCE_DATE_EPOCH="$source_date_epoch" \
  --build-arg RUST_VERSION="$rust_version" \
  --build-arg CARGO_NEAR_VERSION="$cargo_near_version" \
  --build-arg CARGO_NEAR_SHA256="$cargo_near_sha256" \
  "$@" \
  "$dir"
