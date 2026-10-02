# Architecture decision records

One file per decision that changed the shape of the system, immutable once
accepted. The system as it stands is described in
[ARCHITECTURE.md](../../ARCHITECTURE.md).

| Number | Decision | Status |
| --- | --- | --- |
| [0001](0001-split-cinelist-into-a-rust-api-and-a-static-web-front.md) | Split CineList into a Rust API and a static web front on one origin | accepted |
| [0002](0002-store-data-in-postgresql-managed-by-cloudnativepg.md) | Store data in PostgreSQL, managed by CloudNativePG | accepted |
| [0003](0003-authenticate-with-long-lived-jwts-from-a-cookie-or-a-bearer-header.md) | Authenticate with long-lived JWTs, from a cookie or a bearer header | accepted |
