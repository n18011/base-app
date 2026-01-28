# base-app

Monorepo + Microservices architecture with Rust, Kubernetes, and ArgoCD.

## Quick Start

```bash
# Setup cluster and ArgoCD
make setup

# Build and deploy all services
make dev-all && make deploy-all

# Verify
curl http://localhost:8080                    # base-app
curl -X POST http://localhost:8081/echo \
  -H "Content-Type: application/json" \
  -d '{"message":"World"}'                    # echo-service
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| base-app | 8080 | Main application |
| echo-service | 8081 | Echo mock (returns "Hello {message}") |

## Development

```bash
# Build specific service
make build SERVICE=echo-service

# Full dev cycle (build + load to kind)
make dev SERVICE=echo-service

# Deploy via ArgoCD
git push  # Auto-syncs
```

## Documentation

- [Development Guide](docs/CONTRIB.md) - Setup, workflow, adding services
- [Operations Runbook](docs/RUNBOOK.md) - Deployment, monitoring, troubleshooting

## Project Structure

```
├── services/          # Microservices (Rust)
├── libs/common/       # Shared library
├── docker/            # Dockerfiles per service
├── k8s/               # Kubernetes manifests (Kustomize)
├── argocd/            # ArgoCD Applications
└── kind/              # Local cluster config
```

## Commands

```bash
make help              # Show all commands
make status            # Cluster and pod status
make logs SERVICE=xxx  # View service logs
make argocd-ui         # ArgoCD credentials
```
