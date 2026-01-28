#!/bin/bash

# MCP Context Browser Kubernetes Deployment Script
# Version: 0.1.0

set -e

NAMESPACE=${NAMESPACE:-default}
APP_NAME="mcb"
VERSION="v0.1.1"

echo "🚀 Deploying MCP Context Browser $VERSION to namespace: $NAMESPACE"

# Create namespace if it doesn't exist
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Apply RBAC
echo "🔐 Applying RBAC..."
kubectl apply -f rbac.yaml -n $NAMESPACE

# Apply ConfigMaps
echo "📝 Applying ConfigMaps..."
kubectl apply -f configmap.yaml -n $NAMESPACE

# Apply Secrets (you need to populate these with actual values)
echo "🔑 Applying Secrets (make sure to populate with actual values)..."
kubectl apply -f secrets.yaml -n $NAMESPACE

# Apply Services
echo "🌐 Applying Services..."
kubectl apply -f service.yaml -n $NAMESPACE

# Apply Deployment
echo "🐳 Applying Deployment..."
kubectl apply -f deployment.yaml -n $NAMESPACE

# Apply HPA
echo "📈 Applying HorizontalPodAutoscaler..."
kubectl apply -f hpa.yaml -n $NAMESPACE

# Apply NetworkPolicy
echo "🔒 Applying NetworkPolicy..."
kubectl apply -f networkpolicy.yaml -n $NAMESPACE

# Apply PodDisruptionBudget
echo "🛡️  Applying PodDisruptionBudget..."
kubectl apply -f poddisruptionbudget.yaml -n $NAMESPACE

# Apply ServiceMonitor (if Prometheus is available)
if kubectl api-resources | grep -q servicemonitor; then
    echo "📊 Applying ServiceMonitor..."
    kubectl apply -f servicemonitor.yaml -n $NAMESPACE
else
    echo "⚠️  ServiceMonitor not applied (Prometheus Operator not found)"
fi

# Apply Ingress
echo "🌍 Applying Ingress..."
kubectl apply -f ingress.yaml -n $NAMESPACE

# Wait for rollout
echo "⏳ Waiting for rollout to complete..."
kubectl rollout status deployment/$APP_NAME -n $NAMESPACE --timeout=300s

# Show status
echo "📊 Deployment Status:"
kubectl get pods -l app=$APP_NAME -n $NAMESPACE
kubectl get hpa -l app=$APP_NAME -n $NAMESPACE
kubectl get ingress -l app=$APP_NAME -n $NAMESPACE

echo "✅ MCP Context Browser $VERSION deployed successfully!"
echo ""
echo "🌐 Service URLs:"
echo "  - API: http://$(kubectl get ingress $APP_NAME-ingress -n $NAMESPACE -o jsonpath='{.spec.rules[0].host}')"
echo "  - Metrics: http://$(kubectl get ingress $APP_NAME-ingress -n $NAMESPACE -o jsonpath='{.spec.rules[0].host}'):3001/api/context/metrics"
echo ""
echo "🔧 Useful commands:"
echo "  kubectl logs -f deployment/$APP_NAME -n $NAMESPACE"
echo "  kubectl get events -n $NAMESPACE --sort-by=.metadata.creationTimestamp"
echo "  kubectl describe hpa $APP_NAME-hpa -n $NAMESPACE"