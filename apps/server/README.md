Whaleit Server

Overview
- This crate runs the HTTP API (Axum) and serves static files for the web build.
- It uses the shared `whaleit-core` for all business logic and repositories.
- Database: PostgreSQL only (via `DATABASE_URL`).

Run locally (Rust only)
- From the repo root:
  - `cargo run --manifest-path apps/server/Cargo.toml`

Docker image
- Pull the latest published server image with `docker pull muhx/whaleit:latest`.
- Use that tag (or your locally built image) in the Docker run examples inside the root `README.md`.

Key environment variables
- `WF_LISTEN_ADDR`: Bind address, default `127.0.0.1:8080`.
- `DATABASE_URL`: PostgreSQL connection string (required). Example: `postgres://user:password@localhost:5432/whaleit`.
- `WF_CORS_ALLOW_ORIGINS`: Comma-separated list of allowed origins for CORS. Example: `http://localhost:1420`.
- `WF_REQUEST_TIMEOUT_MS`: Request timeout in milliseconds. Default `30000`.
- `WF_STATIC_DIR`: Directory to serve static assets from (the web build output). Default `dist`.
- `WF_SECRET_KEY`: Required 32-byte key used to encrypt secrets at rest and sign JWTs. Must decode to exactly 32 bytes.
  Can be provided as:
  - Base64-encoded string (recommended): Generate with `openssl rand -base64 32` or `head -c 32 /dev/urandom | base64`
  - 32-byte ASCII string: Must be exactly 32 characters (less secure if contains only printable characters)
  Example: `WF_SECRET_KEY=$(openssl rand -base64 32)`.
- `WF_MULTI_USER_AUTH`: Set to `true` to enable multi-user authentication (register, login, email verification, API keys).
- `WF_AUTH_PASSWORD_HASH`: Enables single-password authentication for web mode when set to an Argon2id PHC string (legacy mode, ignored when `WF_MULTI_USER_AUTH=true`).
- `WF_SMTP_HOST`: SMTP server hostname for sending verification/password-reset emails. When unset, URLs are logged instead.
- `WF_SMTP_PORT`: SMTP port (default: 465).
- `WF_SMTP_USERNAME` / `WF_SMTP_PASSWORD`: SMTP credentials.
- `WF_SMTP_FROM`: From address for outgoing emails (default: `WhaleIt <noreply@localhost>`).
- `WF_APP_URL`: Base URL of the app for email links (default: `http://localhost:8088`).
- `WF_AUTH_TOKEN_TTL_MINUTES`: Optional JWT access token lifetime (minutes). Defaults to `60`.
- `WF_SECRET_FILE`: Optional override for where encrypted secrets are stored. Defaults to `<data-root>/secrets.json`.

Notes
- Database migrations are embedded and applied automatically on startup.
- Secrets in web/server mode are stored in an encrypted JSON file using `WF_SECRET_KEY`.
