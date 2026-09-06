#!/bin/sh
set -eu
claim_data_dir="$(mktemp -d)"
fixture_pid=""

cleanup() {
  if [ -n "$fixture_pid" ]; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$claim_data_dir"
}

trap cleanup EXIT INT TERM

node scripts/manual-check-fixture-proxy.mjs &
fixture_pid=$!
export PORT=4173
export DATA_DIR="$claim_data_dir"
export DATABASE_URL="sqlite://$claim_data_dir/claims.db?mode=rwc"
export FRONTEND_DIR="frontend/dist"
export SESSION_SECRET="claim-suite-session-secret-32-bytes-minimum"
export BILLING_BASE_URL="http://billing.example.com"
export HTTP_PROXY="http://127.0.0.1:4175"
export http_proxy="$HTTP_PROXY"
cargo run --quiet
