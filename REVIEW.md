# REVIEW.md

## What matters in this repository
- Zero-knowledge: never decrypt vault ciphertext on the server.
- Every D1 access to ciphers, folders, sends, devices, and attachments must be scoped to the authenticated user.
- Treat auth, login, registration, PBKDF2 password hashing (≥ 600_000 iterations), JWT secrets, and account/purge flows as high-risk.
- Do not edit bundled `public/web-vault/` except the documented CSS override in `public/css/`.
- Do not add Organizations, sharing, emergency access, or admin unless the PR explicitly asks.
- After writes that notify other devices, keep D1 sessions on `first-primary`.
- Prefer small, explicit fixes over broad refactors.

## Severity calibration
- Critical: cross-user vault leak, attachment ACL bypass, plaintext password or secret in logs, JWT confusion.
- Warning: rate-limit binding regressions (fail-open), missing validation, migration without backup notes.
- Do not flag `rustfmt` or `clippy --target wasm32-unknown-unknown` issues that tooling already enforces.

## Verification expectations
- D1 changes need a file in `migrations/` and a check against `sql/schema.sql`.
- Call out new env vars, secrets, and R2/DO/ratelimit bindings in `wrangler.toml`.
- Auth, crypto, and isolation changes should be reviewed against the observable Bitwarden-client result, even when unit tests are still sparse.
