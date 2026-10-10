#!/usr/bin/env bash
#
# scripts/cf-deploy.sh  --  DEPLOY phase for Cloudflare Workers Builds.
#
# Set as the dashboard's Deploy command:
#   Workers & Pages > warden-worker > Settings > Build
#   Deploy command:  bash scripts/cf-deploy.sh
#
# Runs in the deploy phase, where the auto-generated Workers Builds *build
# token* is the active credential. We reuse that single token for both the D1
# migrations and the deploy, so NO separate CLOUDFLARE_API_TOKEN is needed.
#
#   ONE-TIME SETUP: the build token does not include D1 by default. Add it:
#   My Profile > API Tokens > edit the auto-created "Workers Builds" token
#   > add  Account > D1 > Edit.  (Workers Scripts/KV/R2 are already included.)
#   Without this, the migration step below fails with an authorization error.
#
# This runs the D1 migrations and seeds global domains, then `wrangler deploy` (which
# incrementally re-runs worker-build via wrangler.toml's [build], then uploads).
#
# Build variables (Settings > Build > "Variables and Secrets"):
#   WRANGLER_VERSION      (optional)  pinned wrangler (default below)
#   SEED_GLOBAL_DOMAINS   (optional)  "false" to skip seeding global equivalent domains
#   GLOBAL_DOMAINS_URL    (optional)  pin a specific global_domains.json source
#   SKIP_D1               (optional)  "1" to skip all D1 migrations/seed steps
#                                     (deploy still runs; apply migrations yourself)
#   CLOUDFLARE_ACCOUNT_ID (optional)  set only if wrangler cannot infer the account
#
set -euo pipefail

# Run from the repository root regardless of where the script is invoked.
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

# cf-build.sh installed the Rust toolchain into $HOME/.cargo (shared workspace);
# put it on PATH so `wrangler deploy` can re-run the WASM compile via [build].
export PATH="$HOME/.cargo/bin:$PATH"

# Pin to the toolchain directly so wrangler's worker-build re-run bypasses
# rust-toolchain.toml and does not re-download its clippy/rustfmt components.
if [ -f rust-toolchain.toml ]; then
  RUSTUP_TOOLCHAIN="$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' rust-toolchain.toml | head -n 1)"
  export RUSTUP_TOOLCHAIN
fi

WRANGLER_VERSION="${WRANGLER_VERSION:-4.82.1}"
D1_NAME="${D1_NAME:-vault1}"
SEED_GLOBAL_DOMAINS="${SEED_GLOBAL_DOMAINS:-true}"
SKIP_D1="${SKIP_D1:-0}"

WRANGLER="npx --yes wrangler@${WRANGLER_VERSION}"

step() { printf '\n\033[1m=== %s ===\033[0m\n' "$1"; }

# ---------------------------------------------------------------------------
if [ "${SKIP_D1}" = "1" ]; then
  step "D1 migrations -- SKIPPED (SKIP_D1=1)"
else
  step "D1: apply migrations"
  # Non-interactive by design: in CI wrangler's confirm() prints the prompt and
  # returns its fallback value (yes) without reading stdin, so this never hangs.
  # On an empty database wrangler creates the d1_migrations table and applies
  # every migration in order; on an existing one only unapplied migrations run.
  if ! $WRANGLER d1 migrations apply "${D1_NAME}" --remote; then
    echo "ERROR: 'wrangler d1 migrations apply' failed." >&2
    echo "       If this is an authorization error, add D1:Edit to the Workers" >&2
    echo "       Builds build token (My Profile > API Tokens)." >&2
    exit 1
  fi

  if [ "${SEED_GLOBAL_DOMAINS}" = "false" ]; then
    step "Seed global domains -- SKIPPED (SEED_GLOBAL_DOMAINS=false)"
  else
    step "Seed global equivalent domains"
    if [ -n "${GLOBAL_DOMAINS_URL:-}" ]; then
      bash scripts/db/seed-global-domains.sh --db "${D1_NAME}" --remote \
        --wrangler-version "${WRANGLER_VERSION}" --url "${GLOBAL_DOMAINS_URL}"
    else
      bash scripts/db/seed-global-domains.sh --db "${D1_NAME}" --remote \
        --wrangler-version "${WRANGLER_VERSION}"
    fi
  fi
fi

# ---------------------------------------------------------------------------
step "Deploy Worker (wrangler deploy)"
$WRANGLER deploy