#!/bin/sh
set -eu
claim_data_dir="$(mktemp -d)"
export PORT=4173
export DATA_DIR="$claim_data_dir"
export DATABASE_URL="sqlite://$claim_data_dir/claims.db?mode=rwc"
export FRONTEND_DIR="frontend/dist"
export SESSION_SECRET="claim-suite-session-secret-32-bytes-minimum"
cargo run --quiet
