# Operations Runbook

## Service Endpoints

| Service | Internal Port | NodePort | URL |
|---------|--------------|----------|-----|
| base-app | 8080 | 30080 | http://localhost:8080 |
| echo-service | 8081 | 30081 | http://localhost:8081 |
| ArgoCD UI | 8080 | 30443 | https://localhost:8443 |

## Deployment

### Initial Deployment

```bash
# Full setup from scratch
make clean && make setup
make dev-all
make deploy-all
```

### Update Deployment

```bash
# Option 1: GitOps (recommended)
git add . && git commit -m "change" && git push
# ArgoCD auto-syncs within 3 minutes

# Option 2: Manual sync
make dev-all
make sync-all
```

### Deploy Single Service

```bash
make dev SERVICE=echo-service
make sync SERVICE=echo-service
```

## Monitoring

### Check Cluster Status

```bash
make status
```

### Check Pod Status

```bash
kubectl get pods -A
kubectl get pods -l app=base-app
kubectl get pods -l app=echo-service
```

### View Logs

```bash
# Stream logs
make logs SERVICE=base-app
make logs SERVICE=echo-service

# With kubectl
kubectl logs -l app=echo-service -f --tail=100
```

### ArgoCD UI

```bash
make argocd-ui
# URL: https://localhost:8443
# Username: admin
# Password: (displayed by command)
```

### Check ArgoCD Application Status

```bash
kubectl get applications -n argocd
kubectl describe application base-app -n argocd
kubectl describe application echo-service -n argocd
```

## Common Issues

### Pod CrashLoopBackOff

**Symptoms:** Pod repeatedly restarts

**Diagnosis:**
```bash
kubectl describe pod -l app=<service>
kubectl logs -l app=<service> --previous
```

**Common causes:**
1. Image not loaded: `make load SERVICE=<service>`
2. Health check failing: Check `/health` endpoint
3. Port conflict: Verify containerPort matches service

### ArgoCD Out of Sync

**Symptoms:** Application shows "OutOfSync"

**Fix:**
```bash
# Force refresh
kubectl patch application <app> -n argocd \
  --type merge -p '{"metadata":{"annotations":{"argocd.argoproj.io/refresh":"hard"}}}'

# Or via make
make sync SERVICE=<service>
```

### Image Not Found

**Symptoms:** `ErrImageNeverPull` or `ImagePullBackOff`

**Fix:**
```bash
# Rebuild and reload
make dev SERVICE=<service>

# Verify image exists in kind
docker exec base-app-control-plane crictl images | grep <service>
```

### Port Already in Use

**Symptoms:** Cannot access service on expected port

**Diagnosis:**
```bash
# Check what's using the port
lsof -i :8080
lsof -i :8081

# Check NodePort services
kubectl get svc -A | grep NodePort
```

**Fix:** Delete cluster and recreate
```bash
make clean && make setup
```

### ArgoCD Cannot Connect to Repository

**Symptoms:** Application shows "ComparisonError"

**Diagnosis:**
```bash
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-repo-server
```

**Fix:**
```bash
# Ensure repo is public or add credentials
# Force hard refresh
kubectl patch application <app> -n argocd \
  --type merge -p '{"metadata":{"annotations":{"argocd.argoproj.io/refresh":"hard"}}}'
```

## Rollback

### Rollback via Git

```bash
# Find previous commit
git log --oneline

# Revert to previous commit
git revert HEAD
git push

# ArgoCD will auto-sync to previous state
```

### Rollback via ArgoCD

```bash
# List sync history
kubectl get applications base-app -n argocd -o yaml | grep -A 20 history

# Sync to specific revision
kubectl patch application base-app -n argocd --type merge -p \
  '{"operation": {"sync": {"revision": "<commit-sha>"}}}'
```

### Emergency: Delete and Recreate

```bash
# Delete application (keeps pods running briefly)
kubectl delete application <app> -n argocd

# Recreate
kubectl apply -f argocd/applications/<app>.yaml
```

## Cleanup

### Delete Single Application

```bash
kubectl delete -f argocd/applications/<service>.yaml
```

### Delete All Applications

```bash
make clean-app
```

### Delete ArgoCD Only

```bash
make clean-argocd
```

### Full Cleanup (Delete Cluster)

```bash
make clean
```

## Scaling

### Manual Scaling

```bash
kubectl scale deployment base-app --replicas=3
kubectl scale deployment echo-service --replicas=2
```

### Persistent Scaling (via Kustomize)

Edit `k8s/overlays/dev/<service>/kustomization.yaml`:
```yaml
replicas:
  - name: <service>
    count: 3
```

Then push to Git for ArgoCD sync.
