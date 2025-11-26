{{/*
Expand the name of the chart.
*/}}
{{- define "gjallarhorn.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "gjallarhorn.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "gjallarhorn.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "gjallarhorn.labels" -}}
helm.sh/chart: {{ include "gjallarhorn.chart" . }}
{{ include "gjallarhorn.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- with .Values.commonLabels }}
{{ toYaml . }}
{{- end }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "gjallarhorn.selectorLabels" -}}
app.kubernetes.io/name: {{ include "gjallarhorn.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "gjallarhorn.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "gjallarhorn.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the namespace
*/}}
{{- define "gjallarhorn.namespace" -}}
{{- if .Values.namespaceOverride }}
{{- .Values.namespaceOverride }}
{{- else }}
{{- .Release.Namespace }}
{{- end }}
{{- end }}

{{/*
PostgreSQL cluster name
*/}}
{{- define "gjallarhorn.postgres.name" -}}
{{- printf "%s-postgres" (include "gjallarhorn.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
PostgreSQL connection string
*/}}
{{- define "gjallarhorn.postgres.connectionString" -}}
{{- if .Values.postgresql.externalDatabase.enabled }}
{{- .Values.postgresql.externalDatabase.connectionString }}
{{- else }}
{{- printf "postgresql://%s:%s@%s-rw:%d/%s" .Values.postgresql.auth.username .Values.postgresql.auth.password (include "gjallarhorn.postgres.name" .) (int .Values.postgresql.auth.port) .Values.postgresql.auth.database }}
{{- end }}
{{- end }}

{{/*
Image name
*/}}
{{- define "gjallarhorn.image" -}}
{{- if .Values.image.digest }}
{{- printf "%s@%s" .Values.image.repository .Values.image.digest }}
{{- else }}
{{- printf "%s:%s" .Values.image.repository (default .Chart.AppVersion .Values.image.tag) }}
{{- end }}
{{- end }}

{{/*
Keycloak URL
*/}}
{{- define "gjallarhorn.keycloak.url" -}}
{{- if .Values.keycloak.externalUrl }}
{{- .Values.keycloak.externalUrl }}
{{- else }}
{{- printf "http://%s.%s.svc.cluster.local:%d/realms/%s" .Values.keycloak.serviceName .Values.keycloak.namespace (int .Values.keycloak.port) .Values.keycloak.realm }}
{{- end }}
{{- end }}
