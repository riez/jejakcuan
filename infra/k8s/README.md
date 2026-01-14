# JejakCuan Kubernetes Deployment

Production-ready Kubernetes configuration for JejakCuan.

## Prerequisites

- Kubernetes cluster (1.25+)
- kubectl configured
- cert-manager installed
- nginx-ingress controller
- StorageClass named "standard"

## Quick Start

```bash
# Create namespace and deploy
kubectl apply -k infra/k8s/

# Check deployment status
kubectl -n jejakcuan get pods

# View logs
kubectl -n jejakcuan logs -f deployment/jejakcuan-api
```

## Configuration

### Secrets

Before deploying, update `secrets.yaml` with actual values:

```bash
# Generate secure secrets
openssl rand -base64 32  # For JWT_SECRET

# Update secrets
kubectl -n jejakcuan create secret generic jejakcuan-secrets \
  --from-literal=DATABASE_URL="postgresql://..." \
  --from-literal=REDIS_URL="redis://..." \
  --from-literal=JWT_SECRET="..." \
  --dry-run=client -o yaml | kubectl apply -f -
```

### Domain Setup

1. Update `ingress.yaml` with your domain
2. Update email in ClusterIssuer
3. Configure DNS A records pointing to ingress IP

## Architecture

```
                    ┌─────────────┐
                    │   Ingress   │
                    │   (nginx)   │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
   │   Web   │        │   API   │        │   ML    │
   │ (3 pod) │        │ (3 pod) │        │ (2 pod) │
   └─────────┘        └────┬────┘        └────┬────┘
                           │                  │
        ┌──────────────────┴──────────────────┘
        │                  │
   ┌────▼────┐        ┌────▼────┐
   │ Postgres│        │  Redis  │
   │ (1 pod) │        │ (1 pod) │
   └─────────┘        └─────────┘
```

## Scaling

HPA is configured for the API deployment:
- Min replicas: 3
- Max replicas: 10
- Target CPU: 70%
- Target Memory: 80%

Manual scaling:
```bash
kubectl -n jejakcuan scale deployment/jejakcuan-api --replicas=5
```

## Monitoring

### Health Checks

```bash
# API health
kubectl -n jejakcuan exec -it deploy/jejakcuan-api -- curl localhost:3000/health

# Database connectivity
kubectl -n jejakcuan exec -it deploy/postgres-0 -- pg_isready -U jejakcuan

# Redis connectivity
kubectl -n jejakcuan exec -it deploy/redis-0 -- redis-cli ping
```

### Logs

```bash
# All API logs
kubectl -n jejakcuan logs -f -l app=jejakcuan-api

# Follow specific pod
kubectl -n jejakcuan logs -f jejakcuan-api-xxx

# Previous logs (after restart)
kubectl -n jejakcuan logs --previous jejakcuan-api-xxx
```

## Security

### Network Policies

- Default deny all traffic
- API can access: Postgres, Redis, ML, external HTTPS
- Web can access: API only
- ML can access: Postgres, external HTTPS
- Database/Redis only accessible from internal pods

### Pod Security

- Non-root user (UID 1000)
- Read-only root filesystem
- No privilege escalation
- Resource limits enforced

## Backup

### Database Backup

```bash
# Create backup
kubectl -n jejakcuan exec postgres-0 -- pg_dump -U jejakcuan jejakcuan > backup.sql

# Restore
kubectl -n jejakcuan exec -i postgres-0 -- psql -U jejakcuan jejakcuan < backup.sql
```

### Redis Backup

```bash
# Trigger BGSAVE
kubectl -n jejakcuan exec redis-0 -- redis-cli -a $REDIS_PASSWORD BGSAVE

# Copy RDB file
kubectl -n jejakcuan cp redis-0:/data/dump.rdb ./redis-backup.rdb
```

## Troubleshooting

### Pod not starting

```bash
kubectl -n jejakcuan describe pod <pod-name>
kubectl -n jejakcuan logs <pod-name>
```

### Database connection issues

```bash
# Test from API pod
kubectl -n jejakcuan exec -it deploy/jejakcuan-api -- \
  psql $DATABASE_URL -c "SELECT 1"
```

### Certificate issues

```bash
kubectl describe certificate -n jejakcuan
kubectl describe certificaterequest -n jejakcuan
```
