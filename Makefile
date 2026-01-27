.PHONY: help prerequisites cluster argocd setup build load deploy dev status logs argocd-ui sync clean-app clean-argocd clean

# Configuration
CLUSTER_NAME := base-app
IMAGE_NAME := base-app
IMAGE_TAG := latest
ARGOCD_NAMESPACE := argocd

# Colors for help
YELLOW := \033[33m
GREEN := \033[32m
CYAN := \033[36m
RESET := \033[0m

##@ Help
help: ## Display this help
	@echo ""
	@echo "Usage: make [target]"
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
	@echo "  1. make build    - Build the application image"
	@echo "  2. make load     - Load image into kind"
	@echo "  3. make deploy   - Deploy via ArgoCD"

##@ Development
build: ## Build Docker image
	@echo "Building Docker image $(IMAGE_NAME):$(IMAGE_TAG)..."
	@docker build -t $(IMAGE_NAME):$(IMAGE_TAG) -f docker/Dockerfile .

load: ## Load image into kind cluster
	@echo "Loading image into kind cluster..."
	@kind load docker-image $(IMAGE_NAME):$(IMAGE_TAG) --name $(CLUSTER_NAME)
	@echo "Image loaded successfully!"

deploy: ## Apply ArgoCD Application
	@echo "Deploying ArgoCD Application..."
	@kubectl apply -f argocd/applications/base-app.yaml
	@echo "Application deployed! Check status with: make status"

dev: build load ## Development cycle (build + load)
	@echo ""
	@echo "Development build complete!"
	@echo "Run 'make sync' to trigger ArgoCD sync if auto-sync is disabled"

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

logs: ## Show application logs
	@kubectl logs -l app=base-app -f --tail=100

argocd-ui: ## Show ArgoCD UI URL and password
	@echo ""
	@echo "=== ArgoCD UI ==="
	@echo "URL:      https://localhost:8443"
	@echo "Username: admin"
	@echo -n "Password: "
	@kubectl -n $(ARGOCD_NAMESPACE) get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" 2>/dev/null | base64 -d || echo "(not ready yet)"
	@echo ""

sync: ## Trigger ArgoCD sync manually
	@echo "Triggering ArgoCD sync..."
	@kubectl patch application base-app -n $(ARGOCD_NAMESPACE) --type merge -p '{"operation": {"sync": {}}}'
	@echo "Sync triggered!"

##@ Cleanup
clean-app: ## Delete application only
	@echo "Deleting application..."
	@kubectl delete -f argocd/applications/base-app.yaml --ignore-not-found
	@kubectl delete -f k8s/overlays/dev --ignore-not-found

clean-argocd: ## Delete ArgoCD only
	@echo "Deleting ArgoCD..."
	@kubectl delete -n $(ARGOCD_NAMESPACE) -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml --ignore-not-found
	@kubectl delete namespace $(ARGOCD_NAMESPACE) --ignore-not-found

clean: ## Delete everything (cluster)
	@echo "Deleting kind cluster '$(CLUSTER_NAME)'..."
	@kind delete cluster --name $(CLUSTER_NAME)
	@echo "Cleanup complete!"
