# Development Guide

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Docker | 20.10+ | Container runtime |
| kind | 0.20+ | Local Kubernetes cluster |
| kubectl | 1.28+ | Kubernetes CLI |
| Rust | 1.75+ | Application development |

Check prerequisites:
```bash
make prerequisites
```

## Project Structure

```
base-app/
├── Cargo.toml                 # Workspace definition
├── Makefile                   # Development commands
├── services/                  # Microservices
│   ├── base-app/             # Main application
│   └── echo-service/         # Echo mock service
├── libs/                      # Shared libraries
│   └── common/               # Common types & utilities
├── docker/                    # Dockerfiles
│   ├── base-app.Dockerfile
│   └── echo-service.Dockerfile
├── k8s/                       # Kubernetes manifests
│   ├── base/                 # Base manifests
│   │   ├── base-app/
│   │   └── echo-service/
│   └── overlays/dev/         # Dev environment
├── argocd/                    # ArgoCD Applications
│   └── applications/
├── kind/                      # kind cluster config
└── docs/                      # Documentation
```

## Development Workflow

### Initial Setup

```bash
# 1. Create cluster and install ArgoCD
make setup

# 2. Build all service images
make build-all

# 3. Load images into kind
make load-all

# 4. Deploy via ArgoCD
make deploy-all
```

### Daily Development

```bash
# Build and deploy specific service
make dev SERVICE=echo-service

# Build and deploy all services
make dev-all

# Push changes (ArgoCD auto-syncs)
git add . && git commit -m "feat: your change" && git push
```

### Available Commands

| Command | Description |
|---------|-------------|
| `make help` | Show all available commands |
| `make prerequisites` | Check required tools |
| `make setup` | Full setup (cluster + ArgoCD) |
| `make build` | Build Docker image for SERVICE |
| `make build-all` | Build all service images |
| `make load` | Load SERVICE image into kind |
| `make load-all` | Load all images into kind |
| `make deploy` | Deploy SERVICE via ArgoCD |
| `make deploy-all` | Deploy all services |
| `make dev` | Build + load for SERVICE |
| `make dev-all` | Build + load all services |
| `make status` | Show cluster and app status |
| `make logs` | Show logs for SERVICE |
| `make sync` | Trigger ArgoCD sync for SERVICE |
| `make sync-all` | Trigger sync for all services |
| `make clean` | Delete cluster |

### SERVICE Variable

Single-service commands accept `SERVICE=xxx`:
```bash
make build SERVICE=echo-service
make logs SERVICE=base-app
make sync SERVICE=echo-service
```

## Adding a New Service

1. Create service directory:
```bash
mkdir -p services/new-service/src
```

2. Create `services/new-service/Cargo.toml`:
```toml
[package]
name = "new-service"
version.workspace = true
edition.workspace = true

[dependencies]
common = { path = "../../libs/common" }
axum = { workspace = true }
tokio = { workspace = true }
# ... other dependencies
```

3. Add to workspace in root `Cargo.toml`:
```toml
[workspace]
members = [
    "services/base-app",
    "services/echo-service",
    "services/new-service",  # Add this
    "libs/common",
]
```

4. Create Dockerfile at `docker/new-service.Dockerfile`

5. Create Kubernetes manifests:
```bash
mkdir -p k8s/base/new-service k8s/overlays/dev/new-service
```

6. Create ArgoCD Application at `argocd/applications/new-service.yaml`

7. Add to Makefile SERVICES variable:
```makefile
SERVICES := base-app echo-service new-service
```

## Testing

### Local Testing (without cluster)

```bash
# Run tests for all services
cargo test --workspace

# Run specific service
cargo test -p echo-service

# Run with logs
RUST_LOG=debug cargo run -p echo-service
```

### Integration Testing (with cluster)

```bash
# Verify base-app
curl http://localhost:8080

# Verify echo-service
curl -X POST http://localhost:8081/echo \
  -H "Content-Type: application/json" \
  -d '{"message":"test"}'

# Check health endpoints
curl http://localhost:8080/health
curl http://localhost:8081/health
```

## Shared Library (libs/common)

The `libs/common` crate provides shared functionality:

- `init_tracing()` - Initialize logging
- `HealthResponse` - Standard health check response
- `ErrorResponse` - Standard error response

Usage in services:
```rust
use common::{init_tracing, HealthResponse};

fn main() {
    common::init_tracing();
    // ...
}
```
