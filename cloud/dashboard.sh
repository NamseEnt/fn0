#!/bin/bash

# Kubernetes Dashboard Access Script
# Uses kubectl port-forward to access the Kubernetes Dashboard
# kubeconfig is fetched from Pulumi and never saved to disk

set -e

NAMESPACE="kubernetes-dashboard"
TARGET_SERVICE="kubernetes-dashboard-kong-proxy" 

echo "=========================================="
echo "Kubernetes Dashboard Access"
echo "=========================================="
echo ""

echo "Fetching kubeconfig from Pulumi..."
KUBECONFIG_CONTENT=$(pulumi stack output kubeconfig --show-secrets 2>/dev/null)

if [ -z "$KUBECONFIG_CONTENT" ]; then
    echo "Error: Could not fetch kubeconfig from Pulumi."
    echo "Make sure you're in the correct directory and the stack exists."
    exit 1
fi

echo "Kubeconfig fetched successfully (not saved to disk)"
echo ""

SERVICE_PORT=443

# Get the admin token using process substitution
echo "Fetching admin token..."
TOKEN=$(kubectl --kubeconfig=<(echo "$KUBECONFIG_CONTENT") get secret dashboard-admin-token -n "$NAMESPACE" -o jsonpath='{.data.token}' 2>/dev/null | base64 -d)

if [ -z "$TOKEN" ]; then
    echo "Error: Could not fetch admin token. Make sure the dashboard is deployed."
    exit 1
fi

LOCAL_PORT="8443"

echo ""
echo "=========================================="
echo "Dashboard Access Information"
echo "=========================================="
echo ""
echo "Dashboard URL: https://localhost:$LOCAL_PORT"
echo ""
echo "Login Token (copy this):"
echo "----------------------------------------"
echo "$TOKEN"
echo "----------------------------------------"
echo ""

# Copy token to clipboard if pbcopy is available
if command -v pbcopy &> /dev/null; then
    echo "$TOKEN" | pbcopy
    echo "✓ Token copied to clipboard!"
    echo ""
fi

echo "Port-forward command (if you need to run manually):"
echo "kubectl port-forward -n $NAMESPACE svc/$TARGET_SERVICE $LOCAL_PORT:$SERVICE_PORT"
echo ""
echo "=========================================="
echo ""
echo "Starting port-forward to dashboard..."
echo "Press Ctrl+C to stop"
echo ""

# Start kubectl port-forward using process substitution (no file saved to disk)
kubectl --kubeconfig=<(echo "$KUBECONFIG_CONTENT") port-forward -n "$NAMESPACE" svc/"$TARGET_SERVICE" "$LOCAL_PORT:$SERVICE_PORT" &
PORT_FORWARD_PID=$!

echo -n "Waiting for Dashboard to become ready..."
MAX_RETRIES=30
COUNT=0
DASHBOARD_URL="https://localhost:$LOCAL_PORT"

while ! curl -k -s -o /dev/null "$DASHBOARD_URL"; do
    sleep 1
    COUNT=$((COUNT+1))
    echo -n "."
    
    if [ $COUNT -ge $MAX_RETRIES ]; then
        echo ""
        echo "Error: Timeout waiting for dashboard to start."
        kill $PORT_FORWARD_PID
        exit 1
    fi
done

echo ""
echo "✓ Dashboard is Ready!"
echo ""

if command -v open &> /dev/null; then
    open "https://localhost:$LOCAL_PORT"
    echo "✓ Browser opened automatically!"
    echo "  (Note: Accept the self-signed certificate warning in your browser)"
    echo ""
fi

wait $PORT_FORWARD_PID
