# cinelist

![Version: 0.1.0](https://img.shields.io/badge/Version-0.1.0-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 0.1.0](https://img.shields.io/badge/AppVersion-0.1.0-informational?style=flat-square)

Film lists and the elimination rounds that pick what to watch next.

Deploys the CineList API and web front, a CloudNativePG cluster for the
database, and an Ingress or an HTTPRoute sending `/api` to the API and the rest
to the front. How to install it is in the
[project README](../../README.md#deployment).

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| api.admin.existingSecret.key | string | `"ADMIN_PASSWORD"` | Key of that Secret holding the password. |
| api.admin.existingSecret.name | string | `nil` | Name of a pre-existing Secret holding the administrator password. Set → takes precedence over `password`. |
| api.admin.password | string | `nil` | Password of the administrator, stored in a generated Secret. Required unless `existingSecret.name` is set. |
| api.admin.username | string | `"admin"` | Username of the administrator, created or updated at startup. |
| api.caCerts.existing.key | string | `"ca-certificates.crt"` | Key in the resource holding the PEM bundle (also the mounted file name). |
| api.caCerts.existing.kind | string | `"ConfigMap"` | Kind of the resource: `ConfigMap` or `Secret`. |
| api.caCerts.existing.name | string | `nil` | Name of the resource. Set → mounts from it instead of generating a Secret from `inline`. |
| api.caCerts.inline | string | `nil` | Inline PEM bundle. Stored in a Secret and mounted. Ignored when `existing.name` is set. |
| api.cookieSecure | bool | `true` | Mark the session cookie `Secure`. Disable only when the site is served over plain HTTP. |
| api.database.existingSecret.key | string | `"DATABASE_URL"` | Key of that Secret holding the connection string. |
| api.database.existingSecret.name | string | `nil` | Name of a pre-existing Secret holding the connection string. Set → takes precedence over `url`. |
| api.database.maxConnections | int | `10` | Largest number of pooled database connections per pod. |
| api.database.url | string | `nil` | Postgres connection string, stored in a generated Secret. Ignored when `postgres.enabled` is true or `existingSecret.name` is set. |
| api.extraEnv | list | `[]` | Extra environment variables of the API container. |
| api.extraVolumeMounts | list | `[]` | Extra volume mounts of the API container. |
| api.extraVolumes | list | `[]` | Extra volumes of the API pods. |
| api.image.repository | string | `"leroyguillaume/cinelist-api"` | Image repository. |
| api.image.tag | string | `nil` | Image tag. Defaults to the chart's `appVersion`. |
| api.invitationTtlDays | int | `7` | Days an invitation link stays valid. |
| api.jwt.existingSecret.key | string | `"JWT_SECRET"` | Key of that Secret holding the signing secret. |
| api.jwt.existingSecret.name | string | `nil` | Name of a pre-existing Secret holding the JWT signing secret. Set → takes precedence over `secret`. |
| api.jwt.secret | string | `nil` | Secret signing the JWTs, at least 32 bytes, stored in a generated Secret. Required unless `existingSecret.name` is set. Changing it signs everyone out. |
| api.logFilter | string | `"info"` | `tracing` filter directive (e.g. `info`, `cinelist_api=debug`). |
| api.replicaCount | int | `1` | Number of API pods. |
| api.resources.limits.ephemeral-storage | string | `"64Mi"` | Ephemeral storage limit. |
| api.resources.limits.memory | string | `"128Mi"` | Memory limit. |
| api.resources.requests.cpu | string | `"20m"` | CPU request. |
| api.resources.requests.ephemeral-storage | string | `"16Mi"` | Ephemeral storage request. |
| api.resources.requests.memory | string | `"32Mi"` | Memory request. |
| api.service.port | int | `8080` | Port of the API Service. |
| api.tokenTtlDays | int | `30` | Days an issued JWT stays valid. |
| global.affinity | object | `{}` | Affinity of every pod. |
| global.host | string | `nil` | Hostname the Ingress and the HTTPRoute answer on. |
| global.image.pullPolicy | string | `"IfNotPresent"` | Image pull policy. |
| global.image.registry | string | `"ghcr.io"` | Registry the images are pulled from. |
| global.imagePullSecrets | list | `[]` | Secrets used to pull the images. |
| global.nodeSelector | object | `{}` | Node selector of every pod. |
| global.podAnnotations | object | `{}` | Annotations added to every pod. |
| global.podSecurityContext.fsGroup | int | `65532` | Group owning mounted volumes. |
| global.podSecurityContext.runAsGroup | int | `65532` | Group the containers run as. |
| global.podSecurityContext.runAsNonRoot | bool | `true` | Run every container as a non-root user. |
| global.podSecurityContext.runAsUser | int | `65532` | User the containers run as; matches the images' `USER`. |
| global.podSecurityContext.seccompProfile.type | string | `"RuntimeDefault"` | Seccomp profile type. |
| global.revisionHistoryLimit | int | `3` | Number of old ReplicaSets each Deployment keeps for rollback. |
| global.securityContext.allowPrivilegeEscalation | bool | `false` | Forbid privilege escalation. |
| global.securityContext.capabilities.drop | list | `["ALL"]` | Capabilities dropped from every container. |
| global.securityContext.readOnlyRootFilesystem | bool | `true` | Mount the root filesystem read-only; neither server writes to disk. |
| global.tolerations | list | `[]` | Tolerations of every pod. |
| httpRoute.annotations | object | `{}` | Annotations of the HTTPRoute. |
| httpRoute.enabled | bool | `false` | Create the HTTPRoute. |
| httpRoute.gateway.name | string | `nil` | Name of the existing Gateway to attach to. Required when enabled. |
| httpRoute.gateway.namespace | string | `nil` | Namespace of that Gateway. Defaults to the release namespace. |
| httpRoute.gateway.sectionName | string | `nil` | Listener of that Gateway to attach to. |
| ingress.annotations | object | `{}` | Annotations of the Ingress. |
| ingress.className | string | `nil` | Ingress class. |
| ingress.enabled | bool | `false` | Create the Ingress. |
| ingress.tls.secretName | string | `nil` | Secret holding the TLS certificate of `host`. Unset → no TLS section. |
| postgres.enabled | bool | `true` | Create the cluster. Disable to use `api.database` instead. |
| postgres.imageName | string | `nil` | Postgres image. Defaults to the operator's. |
| postgres.instances | int | `1` | Number of Postgres instances. |
| postgres.resources.limits.ephemeral-storage | string | `"256Mi"` | Ephemeral storage limit. |
| postgres.resources.limits.memory | string | `"512Mi"` | Memory limit. |
| postgres.resources.requests.cpu | string | `"50m"` | CPU request. |
| postgres.resources.requests.ephemeral-storage | string | `"64Mi"` | Ephemeral storage request. |
| postgres.resources.requests.memory | string | `"256Mi"` | Memory request. |
| postgres.storage.size | string | `"1Gi"` | Size of each instance's volume. |
| postgres.storage.storageClass | string | `nil` | Storage class of the volumes. Defaults to the cluster's. |
| web.extraEnv | list | `[]` | Extra environment variables of the web container. |
| web.extraVolumeMounts | list | `[]` | Extra volume mounts of the web container. |
| web.extraVolumes | list | `[]` | Extra volumes of the web pods. |
| web.image.repository | string | `"leroyguillaume/cinelist-web"` | Image repository. |
| web.image.tag | string | `nil` | Image tag. Defaults to the chart's `appVersion`. |
| web.replicaCount | int | `1` | Number of web pods. |
| web.resources.limits.ephemeral-storage | string | `"64Mi"` | Ephemeral storage limit. |
| web.resources.limits.memory | string | `"64Mi"` | Memory limit. |
| web.resources.requests.cpu | string | `"10m"` | CPU request. |
| web.resources.requests.ephemeral-storage | string | `"16Mi"` | Ephemeral storage request. |
| web.resources.requests.memory | string | `"16Mi"` | Memory request. |
| web.service.port | int | `8080` | Port of the web Service. |
