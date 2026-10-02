# 0002. Store data in PostgreSQL, managed by CloudNativePG

- **Status**: accepted
- **Date**: 2026-09-27

## Context

The API needs durable, relational storage: users, lists, participants, films,
and a history of selections whose eliminations must never be half-written. The
application runs on a Kubernetes cluster that already operates the
CloudNativePG operator. Several API replicas may run at once.

## Decision

We store everything in PostgreSQL. By default the Helm chart creates a
CloudNativePG `Cluster` next to the application and the API reads the
connection string from the Secret the operator generates, verifying the
server certificate against the cluster's own CA. The chart can instead point
the API at an existing database.

## Consequences

- Every change to a list runs in a transaction that locks the list row, so
  concurrent eliminations or edits from several replicas cannot interleave.
- Schema changes are SQL migrations embedded in the API binary and applied at
  startup.
- The chart depends on the CloudNativePG operator being installed; without it,
  `postgres.enabled` must be turned off and a database provided.
- Backups, failover and upgrades of the database are the operator's job, and
  are configured on the `Cluster`, not in the application.
- Local development and the integration tests need a running Postgres
  (`docker-compose.yaml`).

## Alternatives considered

- **SQLite on a volume**: no extra service, but it pins the API to a single
  replica on a single node and makes the volume the backup story.
- **A Postgres pod run by the chart itself**: no operator needed, but
  replication, failover and certificate management would all be ours to write.
