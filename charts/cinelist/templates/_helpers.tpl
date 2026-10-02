{{- define "cinelist.fullname" -}}
{{- if contains .Chart.Name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}

{{- define "cinelist.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
app.kubernetes.io/name: {{ .Chart.Name }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/* Selector labels of a component; call with (dict "ctx" $ "component" "api"). */}}
{{- define "cinelist.selectorLabels" -}}
app.kubernetes.io/name: {{ .ctx.Chart.Name }}
app.kubernetes.io/instance: {{ .ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end -}}

{{/* A component's values merged over `global`; call with (dict "ctx" $ "component" "api"). */}}
{{- define "cinelist.values" -}}
{{- $component := index .ctx.Values .component -}}
{{- toYaml (mergeOverwrite (deepCopy .ctx.Values.global) (deepCopy $component)) -}}
{{- end -}}

{{- define "cinelist.image" -}}
{{- printf "%s/%s:%s" .values.image.registry .values.image.repository (default .ctx.Chart.AppVersion .values.image.tag) -}}
{{- end -}}

{{- define "cinelist.componentName" -}}
{{- printf "%s-%s" (include "cinelist.fullname" .ctx) .component | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "cinelist.postgresName" -}}
{{- printf "%s-postgres" (include "cinelist.fullname" .) | trunc 50 | trimSuffix "-" -}}
{{- end -}}

{{- define "cinelist.adminSecretName" -}}
{{- $secret := .Values.api.admin.existingSecret -}}
{{- if $secret.name -}}
{{- $secret.name -}}
{{- else -}}
{{- printf "%s-admin" (include "cinelist.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "cinelist.adminSecretKey" -}}
{{- .Values.api.admin.existingSecret.key -}}
{{- end -}}

{{- define "cinelist.jwtSecretName" -}}
{{- $secret := .Values.api.jwt.existingSecret -}}
{{- if $secret.name -}}
{{- $secret.name -}}
{{- else -}}
{{- printf "%s-jwt" (include "cinelist.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "cinelist.jwtSecretKey" -}}
{{- .Values.api.jwt.existingSecret.key -}}
{{- end -}}

{{/* The database Secret: the CloudNativePG one, an existing one, or a generated one. */}}
{{- define "cinelist.databaseSecretName" -}}
{{- $secret := .Values.api.database.existingSecret -}}
{{- if $secret.name -}}
{{- $secret.name -}}
{{- else if .Values.postgres.enabled -}}
{{- printf "%s-app" (include "cinelist.postgresName" .) -}}
{{- else -}}
{{- printf "%s-database" (include "cinelist.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "cinelist.databaseSecretKey" -}}
{{- if .Values.api.database.existingSecret.name -}}
{{- .Values.api.database.existingSecret.key -}}
{{- else if .Values.postgres.enabled -}}
uri
{{- else -}}
DATABASE_URL
{{- end -}}
{{- end -}}

{{- define "cinelist.generateDatabaseSecret" -}}
{{- if and (not .Values.api.database.existingSecret.name) (not .Values.postgres.enabled) -}}
true
{{- end -}}
{{- end -}}

{{- define "cinelist.caCertsValidate" -}}
{{- $kind := .Values.api.caCerts.existing.kind -}}
{{- if and .Values.api.caCerts.existing.name (not (has $kind (list "ConfigMap" "Secret"))) -}}
{{- fail (printf "api.caCerts.existing.kind must be ConfigMap or Secret, got %s" $kind) -}}
{{- end -}}
{{- end -}}

{{- define "cinelist.caCertsEnabled" -}}
{{- if or .Values.api.caCerts.inline .Values.api.caCerts.existing.name -}}
true
{{- end -}}
{{- end -}}

{{- define "cinelist.caCertsGenerateSecret" -}}
{{- if and .Values.api.caCerts.inline (not .Values.api.caCerts.existing.name) -}}
true
{{- end -}}
{{- end -}}

{{- define "cinelist.caCertsFromConfigMap" -}}
{{- if and .Values.api.caCerts.existing.name (eq .Values.api.caCerts.existing.kind "ConfigMap") -}}
true
{{- end -}}
{{- end -}}

{{- define "cinelist.caCertsName" -}}
{{- default (printf "%s-ca-certs" (include "cinelist.fullname" .)) .Values.api.caCerts.existing.name -}}
{{- end -}}

{{- define "cinelist.caCertsKey" -}}
{{- .Values.api.caCerts.existing.key -}}
{{- end -}}

{{- define "cinelist.caCertsPath" -}}
/etc/cinelist/ca-certs/{{ include "cinelist.caCertsKey" . }}
{{- end -}}

{{- define "cinelist.host" -}}
{{- required "global.host is required when the Ingress or the HTTPRoute is enabled" .Values.global.host -}}
{{- end -}}
