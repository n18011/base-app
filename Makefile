.PHONY: help prerequisites cluster argocd setup build build-all load load-all deploy deploy-all dev dev-all status logs argocd-ui sync clean-app clean-argocd clean external-secrets infra-deploy infra-status vault-logs postgresql-logs

# Configuration
CLUSTER_NAME := base-app
ARGOCD_NAMESPACE := argocd
SERVICES := base-app echo-service accounting-service
INFRA := vault external-secrets-config postgresql

# Default service (can be overridden with SERVICE=xxx)
SERVICE ?= base-app

# Colors for help
YELLOW := \033[33m
GREEN := \033[32m
CYAN := \033[36m
RESET := \033[0m

##@ Help
help: ## Display this help
	@echo ""
	@echo "Usage: make [target] [SERVICE=xxx]"
	@echo ""
	@echo "Examples:"
	@echo "  make build                  # Build base-app (default)"
	@echo "  make build SERVICE=echo-service"
	@echo "  make build-all              # Build all services"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; section=""} \
		/^##@/ { section=substr($$0, 5); next } \
		/^[a-zA-Z_-]+:.*##/ { \
			if (section != "") { \
				printf "\n$(CYAN)%s:$(RESET)\n", section; \
				section="" \
			} \
			printf "  $(GREEN)%-15s$(RESET) %s\n", $$1, $$2 \
		}' $(MAKEFILE_LIST)
	@echo ""

##@ Setup
prerequisites: ## Check required tools
	@echo "Checking prerequisites..."
	@command -v docker >/dev/null 2>&1 || { echo "docker is required but not installed."; exit 1; }
	@command -v kind >/dev/null 2>&1 || { echo "kind is required but not installed."; exit 1; }
	@command -v kubectl >/dev/null 2>&1 || { echo "kubectl is required but not installed."; exit 1; }
	@echo "Docker:  $$(docker --version)"
	@echo "kind:    $$(kind --version)"
	@echo "kubectl: $$(kubectl version --client --short 2>/dev/null || kubectl version --client | head -1)"
	@echo ""
	@echo "All prerequisites are installed!"

cluster: prerequisites ## Create kind cluster
	@if kind get clusters | grep -q "^$(CLUSTER_NAME)$$"; then \
		echo "Cluster '$(CLUSTER_NAME)' already exists"; \
	else \
		echo "Creating kind cluster '$(CLUSTER_NAME)'..."; \
		kind create cluster --config kind/cluster-config.yaml; \
	fi
	@kubectl cluster-info --context kind-$(CLUSTER_NAME)

argocd: ## Install ArgoCD
	@echo "Installing ArgoCD..."
	@kubectl create namespace $(ARGOCD_NAMESPACE) --dry-run=client -o yaml | kubectl apply -f -
	@kubectl apply -n $(ARGOCD_NAMESPACE) -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
	@echo "Waiting for ArgoCD pods to be ready..."
	@kubectl wait --for=condition=Ready pods --all -n $(ARGOCD_NAMESPACE) --timeout=300s
	@echo ""
	@echo "Patching ArgoCD server for NodePort access..."
	@kubectl patch svc argocd-server -n $(ARGOCD_NAMESPACE) -p '{"spec": {"type": "NodePort", "ports": [{"port": 443, "targetPort": 8080, "nodePort": 30443}]}}'
	@echo ""
	@echo "ArgoCD installed successfully!"
	@$(MAKE) argocd-ui --no-print-directory

setup: cluster argocd ## Full setup (cluster + ArgoCD)
	@echo ""
	@echo "Setup complete! Next steps:"
	@echo "  1. make external-secrets  - Install External Secrets Operator"
	@echo "  2. make infra-deploy      - Deploy infrastructure (Vault, ESO config, PostgreSQL)"
	@echo "  3. make build-all         - Build all service images"
	@echo "  4. make load-all          - Load images into kind"
	@echo "  5. make deploy-all        - Deploy services via ArgoCD"

##@ Development (Single Service)
build: ## Build Docker image for SERVICE
	@echo "Building Docker image $(SERVICE):latest..."
	@docker build -t $(SERVICE):latest -f docker/$(SERVICE).Dockerfile .

load: ## Load SERVICE image into kind cluster
	@echo "Loading $(SERVICE) image into kind cluster..."
	@kind load docker-image $(SERVICE):latest --name $(CLUSTER_NAME)
	@echo "Image loaded successfully!"

deploy: ## Apply ArgoCD Application for SERVICE
	@echo "Deploying ArgoCD Application for $(SERVICE)..."
	@kubectl apply -f argocd/applications/$(SERVICE).yaml
	@echo "Application deployed! Check status with: make status"

dev: build load ## Development cycle for SERVICE (build + load)
	@echo ""
	@echo "Development build complete for $(SERVICE)!"

##@ Development (All Services)
build-all: ## Build all service images
	@for svc in $(SERVICES); do \
		echo "Building $$svc..."; \
		docker build -t $$svc:latest -f docker/$$svc.Dockerfile . || exit 1; \
	done
	@echo "All services built!"

load-all: ## Load all images into kind cluster
	@for svc in $(SERVICES); do \
		echo "Loading $$svc..."; \
		kind load docker-image $$svc:latest --name $(CLUSTER_NAME) || exit 1; \
	done
	@echo "All images loaded!"

deploy-all: ## Apply all ArgoCD Applications
	@echo "Deploying all ArgoCD Applications..."
	@kubectl apply -f argocd/applications/
	@echo "All applications deployed! Check status with: make status"

dev-all: build-all load-all ## Development cycle for all services
	@echo ""
	@echo "Development build complete for all services!"

##@ Infrastructure
external-secrets: ## Install External Secrets Operator via Helm
	@echo "Installing External Secrets Operator..."
	@helm repo add external-secrets https://charts.external-secrets.io 2>/dev/null || true
	@helm repo update
	@helm upgrade --install external-secrets external-secrets/external-secrets \
		-n external-secrets --create-namespace \
		--set installCRDs=true \
		--wait
	@echo "External Secrets Operator installed!"

infra-deploy: ## Deploy infrastructure (Vault, ESO config, PostgreSQL)
	@echo "Deploying infrastructure components..."
	@echo "Step 1: Deploying Vault..."
	@kubectl apply -f argocd/applications/vault.yaml
	@echo "Waiting for Vault to be ready..."
	@sleep 10
	@kubectl wait --for=condition=Ready pod -l app=vault -n vault --timeout=120s 2>/dev/null || echo "Vault pods not ready yet, continuing..."
	@echo ""
	@echo "Step 2: Deploying External Secrets configuration..."
	@kubectl apply -f argocd/applications/external-secrets.yaml
	@echo ""
	@echo "Step 3: Deploying PostgreSQL..."
	@kubectl apply -f argocd/applications/postgresql.yaml
	@echo ""
	@echo "Infrastructure deployment initiated!"
	@echo "Run 'make infra-status' to check status"

infra-status: ## Show infrastructure status
	@echo "=== Vault ==="
	@kubectl get pods -n vault 2>/dev/null || echo "Vault namespace not found"
	@echo ""
	@echo "=== External Secrets ==="
	@kubectl get pods -n external-secrets 2>/dev/null || echo "External Secrets namespace not found"
	@kubectl get clustersecretstores 2>/dev/null || echo "No ClusterSecretStores found"
	@echo ""
	@echo "=== PostgreSQL ==="
	@kubectl get pods -l app=postgresql 2>/dev/null || echo "PostgreSQL not found"
	@kubectl get externalsecrets 2>/dev/null || echo "No ExternalSecrets found"
	@kubectl get secrets postgresql-secret 2>/dev/null || echo "PostgreSQL secret not found"

vault-logs: ## Show Vault logs
	@kubectl logs -l app=vault -n vault -f --tail=100

postgresql-logs: ## Show PostgreSQL logs
	@kubectl logs -l app=postgresql -f --tail=100

##@ Operation
status: ## Show cluster and app status
	@echo "=== Cluster Status ==="
	@kubectl cluster-info --context kind-$(CLUSTER_NAME) 2>/dev/null || echo "Cluster not running"
	@echo ""
	@echo "=== Pods ==="
	@kubectl get pods -A
	@echo ""
	@echo "=== ArgoCD Applications ==="
	@kubectl get applications -n $(ARGOCD_NAMESPACE) 2>/dev/null || echo "No applications found"

logs: ## Show logs for SERVICE
	@kubectl logs -l app=$(SERVICE) -f --tail=100

argocd-ui: ## Show ArgoCD UI URL and password
	@echo ""
	@echo "=== ArgoCD UI ==="
	@echo "URL:      https://localhost:8443"
	@echo "Username: admin"
	@echo -n "Password: "
	@kubectl -n $(ARGOCD_NAMESPACE) get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" 2>/dev/null | base64 -d || echo "(not ready yet)"
	@echo ""

sync: ## Trigger ArgoCD sync for SERVICE
	@echo "Triggering ArgoCD sync for $(SERVICE)..."
	@kubectl patch application $(SERVICE) -n $(ARGOCD_NAMESPACE) --type merge -p '{"operation": {"sync": {}}}'
	@echo "Sync triggered!"

sync-all: ## Trigger ArgoCD sync for all applications
	@for svc in $(SERVICES); do \
		echo "Syncing $$svc..."; \
		kubectl patch application $$svc -n $(ARGOCD_NAMESPACE) --type merge -p '{"operation": {"sync": {}}}' 2>/dev/null || true; \
	done
	@echo "All syncs triggered!"

##@ Cleanup
clean-app: ## Delete all applications
	@echo "Deleting applications..."
	@kubectl delete -f argocd/applications/ --ignore-not-found
	@kubectl delete -f k8s/overlays/dev --ignore-not-found

clean-argocd: ## Delete ArgoCD only
	@echo "Deleting ArgoCD..."
	@kubectl delete -n $(ARGOCD_NAMESPACE) -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml --ignore-not-found
	@kubectl delete namespace $(ARGOCD_NAMESPACE) --ignore-not-found

clean: ## Delete everything (cluster)
	@echo "Deleting kind cluster '$(CLUSTER_NAME)'..."
	@kind delete cluster --name $(CLUSTER_NAME)
	@echo "Cleanup complete!"
