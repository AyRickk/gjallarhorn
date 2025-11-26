# Gjallarhorn Helm Chart

A production-ready Helm chart for deploying Gjallarhorn, a high-performance feedback collection API built with Rust.

## Features

- **Production-Ready**: Comprehensive configuration for production deployments
- **High Availability**: Multi-replica deployment with horizontal pod autoscaling
- **Database Management**: Integrated PostgreSQL with CloudNativePG (CNPG) operator
- **Observability**: Prometheus metrics, ServiceMonitor, and health checks
- **Security**: Network policies, security contexts, and TLS support
- **Scalability**: HPA with custom scaling policies
- **Backup & Recovery**: Automated PostgreSQL backups to S3/Azure/GCS

## Prerequisites

- Kubernetes 1.24+
- Helm 3.8+
- CloudNativePG operator (if using managed PostgreSQL)
- Prometheus Operator (optional, for ServiceMonitor)
- cert-manager (optional, for TLS certificates)
- Ingress controller (optional, for ingress)

## Installing the Chart

### Basic Installation

```bash
# Install with default values
helm install gjallarhorn ./gjallarhorn

# Install in a specific namespace
helm install gjallarhorn ./gjallarhorn -n feedback-system --create-namespace
```

### Production Installation

```bash
# Install with production values
helm install gjallarhorn ./gjallarhorn \
  -n feedback-system \
  --create-namespace \
  -f values-prod.yaml
```

### Custom Installation

```bash
# Override specific values
helm install gjallarhorn ./gjallarhorn \
  --set image.tag=1.0.0 \
  --set postgresql.auth.password=YourSecurePassword \
  --set ingress.hosts[0].host=api.example.com
```

## Configuration

### Key Configuration Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas (if HPA disabled) | `3` |
| `image.repository` | Image repository | `your-registry/feedback-api` |
| `image.tag` | Image tag | `""` (uses appVersion) |
| `image.pullPolicy` | Image pull policy | `Always` |
| `autoscaling.enabled` | Enable horizontal pod autoscaling | `true` |
| `autoscaling.minReplicas` | Minimum replicas | `3` |
| `autoscaling.maxReplicas` | Maximum replicas | `20` |
| `postgresql.enabled` | Enable PostgreSQL CNPG cluster | `true` |
| `postgresql.instances` | Number of PostgreSQL instances | `3` |
| `postgresql.storage.size` | PostgreSQL storage size | `10Gi` |
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class name | `nginx` |
| `networkPolicy.enabled` | Enable network policies | `true` |
| `metrics.enabled` | Enable Prometheus metrics | `true` |

### Application Configuration

```yaml
app:
  host: "0.0.0.0"
  port: 8080
  exportMaxRecords: 10000
  logLevel: "info,feedback_api=debug"
  extraEnv: {}
  extraSecretEnv: {}
```

### PostgreSQL Configuration

```yaml
postgresql:
  enabled: true
  instances: 3
  auth:
    database: feedback_db
    username: feedback_user
    password: "CHANGE_ME_IN_PRODUCTION"
  storage:
    size: 10Gi
    storageClass: ""
  backup:
    enabled: true
    destinationPath: "s3://your-backup-bucket/feedback-postgres"
    retentionPolicy: "30d"
```

### Keycloak Configuration

```yaml
keycloak:
  realm: "master"
  jwksCacheTtl: 3600
  # For external Keycloak
  externalUrl: "https://keycloak.example.com/realms/master"
  # Or for in-cluster Keycloak
  serviceName: "keycloak"
  namespace: "auth-system"
  port: 8080
```

### Ingress Configuration

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
  hosts:
    - host: api.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - hosts:
        - api.example.com
      secretName: gjallarhorn-api-tls
```

## Upgrading

### Upgrade to a new version

```bash
# Upgrade with new values
helm upgrade gjallarhorn ./gjallarhorn \
  -n feedback-system \
  -f values-prod.yaml

# Upgrade with new image version
helm upgrade gjallarhorn ./gjallarhorn \
  --reuse-values \
  --set image.tag=1.1.0
```

### Rollback

```bash
# Rollback to previous release
helm rollback gjallarhorn -n feedback-system

# Rollback to specific revision
helm rollback gjallarhorn 2 -n feedback-system
```

## Uninstalling

```bash
# Uninstall the chart
helm uninstall gjallarhorn -n feedback-system

# To also delete the namespace
kubectl delete namespace feedback-system
```

**Warning**: Uninstalling will delete the PostgreSQL cluster and all data unless you have backups enabled!

## Production Deployment Guide

### 1. Prerequisites

Install required operators:

```bash
# Install CloudNativePG operator
kubectl apply -f \
  https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.22/releases/cnpg-1.22.0.yaml

# Install Prometheus Operator (if not already installed)
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack -n monitoring --create-namespace

# Install cert-manager (if not already installed)
kubectl apply -f \
  https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml
```

### 2. Prepare Secrets

```bash
# Create backup credentials secret
kubectl create secret generic backup-creds \
  -n feedback-system \
  --from-literal=ACCESS_KEY_ID=your-access-key \
  --from-literal=ACCESS_SECRET_KEY=your-secret-key

# Or use Sealed Secrets / External Secrets Operator for production
```

### 3. Customize Production Values

Create a `my-prod-values.yaml`:

```yaml
image:
  repository: your-registry/feedback-api
  tag: "1.0.0"

postgresql:
  auth:
    password: "YourVerySecurePassword"
  storage:
    size: 100Gi
    storageClass: "fast-ssd"
  backup:
    destinationPath: "s3://your-prod-bucket/gjallarhorn"

ingress:
  hosts:
    - host: api.gjallarhorn.io
      paths:
        - path: /
          pathType: Prefix
  tls:
    - hosts:
        - api.gjallarhorn.io
      secretName: gjallarhorn-api-tls

keycloak:
  externalUrl: "https://auth.gjallarhorn.io/realms/production"

webhooks:
  urls:
    - "https://webhooks.example.com/feedback"
```

### 4. Deploy to Production

```bash
# Deploy with production values
helm install gjallarhorn ./gjallarhorn \
  -n feedback-system \
  --create-namespace \
  -f values-prod.yaml \
  -f my-prod-values.yaml

# Verify deployment
kubectl get all -n feedback-system
kubectl get cluster -n feedback-system
```

### 5. Verify Health

```bash
# Check pod status
kubectl get pods -n feedback-system

# Check PostgreSQL cluster
kubectl get cluster -n feedback-system

# Check ingress
kubectl get ingress -n feedback-system

# Test the API
curl https://api.gjallarhorn.io/health
```

## Monitoring & Observability

### Prometheus Metrics

The application exposes metrics at `/metrics`. If you're using Prometheus Operator, a ServiceMonitor is automatically created.

```bash
# View ServiceMonitor
kubectl get servicemonitor -n feedback-system

# Port-forward to view metrics locally
kubectl port-forward -n feedback-system svc/gjallarhorn-metrics 8080:8080
curl http://localhost:8080/metrics
```

### Grafana Dashboards

Import the Grafana dashboards from `docker/grafana/dashboards/` directory.

### Logs

```bash
# View application logs
kubectl logs -n feedback-system -l app.kubernetes.io/name=gjallarhorn -f

# View PostgreSQL logs
kubectl logs -n feedback-system -l cnpg.io/cluster=gjallarhorn-postgres -f
```

## Backup & Recovery

### PostgreSQL Backups

Backups are automatically configured if `postgresql.backup.enabled=true`.

```bash
# View backup status
kubectl get backup -n feedback-system

# Trigger manual backup
kubectl cnpg backup gjallarhorn-postgres -n feedback-system

# View scheduled backups
kubectl get scheduledbackup -n feedback-system
```

### Restore from Backup

To restore from a backup, update the PostgreSQL cluster spec:

```yaml
postgresql:
  bootstrap:
    recovery:
      source: gjallarhorn-postgres
      recoveryTarget:
        targetTime: "2024-01-01 00:00:00"
```

## Troubleshooting

### Common Issues

#### PostgreSQL cluster not starting

```bash
# Check CNPG operator logs
kubectl logs -n cnpg-system -l app.kubernetes.io/name=cloudnative-pg

# Check cluster status
kubectl describe cluster -n feedback-system gjallarhorn-postgres
```

#### Pods not ready

```bash
# Check pod events
kubectl describe pod -n feedback-system <pod-name>

# Check pod logs
kubectl logs -n feedback-system <pod-name>

# Check readiness probe
kubectl exec -n feedback-system <pod-name> -- wget -qO- http://localhost:8080/health
```

#### Ingress not working

```bash
# Check ingress status
kubectl describe ingress -n feedback-system gjallarhorn

# Check ingress controller logs
kubectl logs -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx
```

### Debug Mode

Enable verbose logging:

```bash
helm upgrade gjallarhorn ./gjallarhorn \
  --reuse-values \
  --set app.logLevel="debug,feedback_api=trace"
```

## Security Considerations

### Production Security Checklist

- [ ] Change default PostgreSQL password
- [ ] Use external secret management (Sealed Secrets, External Secrets Operator)
- [ ] Enable network policies
- [ ] Configure TLS for ingress
- [ ] Set up backup encryption
- [ ] Review and customize RBAC if needed
- [ ] Enable pod security policies/standards
- [ ] Configure resource limits and quotas
- [ ] Use private image registry with authentication
- [ ] Enable audit logging
- [ ] Regular security scanning of container images

### Network Policies

Network policies are enabled by default. Customize allowed traffic:

```yaml
networkPolicy:
  enabled: true
  ingress:
    customRules:
      - from:
        - namespaceSelector:
            matchLabels:
              name: custom-namespace
  egress:
    customRules:
      - to:
        - podSelector:
            matchLabels:
              app: custom-app
```

## Development

### Local Testing

```bash
# Template the chart
helm template gjallarhorn ./gjallarhorn -f values.yaml

# Lint the chart
helm lint ./gjallarhorn

# Package the chart
helm package ./gjallarhorn

# Test installation (dry-run)
helm install gjallarhorn ./gjallarhorn --dry-run --debug
```

## Contributing

Contributions are welcome! Please ensure:

1. All templates are properly tested
2. Documentation is updated
3. Follow Helm best practices
4. Maintain backward compatibility

## License

MIT

## Support

For issues and questions:
- GitHub Issues: https://github.com/yourusername/gjallarhorn/issues
- Documentation: https://github.com/yourusername/gjallarhorn/tree/main/helm/gjallarhorn

---

When users speak, Gjallarhorn sounds! 🎺
