# CineList

A small web app to keep film lists with friends and pick tonight's film by
eliminating the others, one turn each, until a single one is left.

## Description

CineList replaces the shared phone note where a group keeps the films it wants
to watch. Each list holds films and a set of participants. When it is time
to watch something, a **selection** starts: participants take turns
eliminating a film from the list until one survives. You then say whether you
watched it — it leaves the list — or whether you could not find it, in which
case the film eliminated last takes its place.

- Films come and go freely, but a selection starts only when every
  participant gets the same number of turns (`films - 1` is a multiple of the
  number of participants); the app says how many films to add otherwise. The
  first player rotates from one selection to the next.
- A selection is played on one device, passed from hand to hand: participants
  are names in a list, not accounts.
- Every selection is kept: who eliminated which film, at which place, and how
  each film fared over time.
- Accounts are created by an administrator and activated through a single-use
  invitation link. There is no sign-up and no e-mail.
- The interface is available in French and English, and designed for phones
  first.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the design and the reasoning behind it.

## Getting started

### Prerequisites

- Docker, for the development database: <https://docs.docker.com/get-started/get-docker/>
- Rust, through `rustup`; the toolchain version is pinned in
  [rust-toolchain.toml](rust-toolchain.toml) and installed on first use:

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- Node.js 26 with npm: <https://nodejs.org/en/download>

### Installation

```sh
git clone https://github.com/leroyguillaume/cinelist.git
```

```sh
cd cinelist
```

```sh
npm ci --prefix web
```

### Configuration

The API is configured through environment variables (or the matching
`--kebab-case` flags, see `cargo run -p cinelist-api -- --help`).

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `DATABASE_URL` | yes | — | Postgres connection string |
| `ADMIN_PASSWORD` | yes | — | Password of the administrator, reset at every start |
| `JWT_SECRET` | yes | — | Secret signing the JWTs (HS256), at least 32 bytes; changing it signs everyone out |
| `ADMIN_USERNAME` | no | `admin` | Username of the administrator, created at start if missing |
| `BIND_ADDR` | no | `0.0.0.0:8080` | Address the HTTP server listens on |
| `DATABASE_MAX_CONNECTIONS` | no | `10` | Size of the database connection pool |
| `TOKEN_TTL_DAYS` | no | `30` | Days an issued JWT stays valid |
| `INVITATION_TTL_DAYS` | no | `7` | Days an invitation link stays valid |
| `COOKIE_SECURE` | no | `true` | Mark the token cookie `Secure`; set `false` only to serve plain HTTP |
| `LOG_FILTER` | no | `info` | `tracing` filter directive, e.g. `cinelist_api=debug` |
| `PGSSLMODE`, `PGSSLROOTCERT` | no | — | Standard Postgres TLS settings, honoured by the database driver |

The web front takes one build-time variable, `VITE_LOG_LEVEL` (`debug`,
`info`, `warn` or `error`), which defaults to `debug` in development and
`warn` in a production build.

For development, [.cargo/config.toml](.cargo/config.toml) provides values
pointing at the `docker-compose` database. Variables already set in your
environment take precedence, and Cargo applies these values only to the
commands it runs.

### Usage

Start the database:

```sh
docker compose up -d --wait
```

Start the API on <http://localhost:8080>:

```sh
cargo run -p cinelist-api
```

In another terminal, start the web front on <http://localhost:5173>; it
forwards `/api` to the API:

```sh
npm --prefix web run dev
```

Sign in as `admin` with the `ADMIN_PASSWORD` from `.cargo/config.toml`, then
create users from the **Users** page and send them their invitation link.

The API reference is served by the API itself, at `/api/docs`, from the OpenAPI
document at `/api/openapi.json`. Signing in with `POST /api/v1/sessions` returns
a JWT, also set as a cookie; clients other than the browser send it as
`Authorization: Bearer <token>`.

### Deployment

The application ships as two images, `ghcr.io/leroyguillaume/cinelist-api` and
`ghcr.io/leroyguillaume/cinelist-web`, and a Helm chart published to
`oci://ghcr.io/leroyguillaume/charts/cinelist`. The chart needs the
[CloudNativePG](https://cloudnative-pg.io) operator for its database, unless
you point it at an existing one. Its values are documented in
[charts/cinelist/README.md](charts/cinelist/README.md).

Create the administrator's password and the JWT signing secret, then install
the chart; this example routes traffic with an HTTPRoute attached to an
existing Gateway:

```sh
kubectl create namespace cinelist
```

```sh
kubectl --namespace cinelist create secret generic cinelist-admin --from-literal=ADMIN_PASSWORD='<password>'
```

```sh
kubectl --namespace cinelist create secret generic cinelist-jwt --from-literal=JWT_SECRET="$(openssl rand -base64 48)"
```

```sh
helm install cinelist oci://ghcr.io/leroyguillaume/charts/cinelist --version 0.1.0 --namespace cinelist --set global.host=<hostname> --set api.admin.existingSecret.name=cinelist-admin --set api.jwt.existingSecret.name=cinelist-jwt --set httpRoute.enabled=true --set httpRoute.gateway.name=<gateway> --set httpRoute.gateway.namespace=<gateway-namespace>
```

Releases are cut with [scripts/release.sh](scripts/release.sh), as described in
[CONTRIBUTING.md](CONTRIBUTING.md#releasing).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0 — see [LICENSE](LICENSE).
