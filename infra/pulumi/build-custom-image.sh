#!/bin/bash
set -euo pipefail

COMPARTMENT_ID="$1"
AD="$2"
SUBNET_ID="$3"
BASE_IMAGE_ID="$4"
CUSTOM_IMAGE_NAME="fn0-ol10-podman-aarch64"

existing=$(oci compute image list \
  --compartment-id "$COMPARTMENT_ID" \
  --display-name "$CUSTOM_IMAGE_NAME" \
  --lifecycle-state AVAILABLE \
  --query 'data[0].id' \
  --raw-output 2>/dev/null || echo "")

if [ -n "$existing" ] && [ "$existing" != "None" ]; then
  echo -n "$existing"
  exit 0
fi

CLOUD_INIT=$(cat <<'CINIT'
#!/bin/bash
dnf install -y podman
touch /tmp/podman-installed
CINIT
)
USER_DATA=$(echo "$CLOUD_INIT" | base64 -w0)

INSTANCE_ID=$(oci compute instance launch \
  --compartment-id "$COMPARTMENT_ID" \
  --availability-domain "$AD" \
  --shape "VM.Standard.A1.Flex" \
  --shape-config '{"ocpus": 1, "memoryInGBs": 6}' \
  --image-id "$BASE_IMAGE_ID" \
  --subnet-id "$SUBNET_ID" \
  --assign-public-ip true \
  --metadata "{\"user_data\": \"$USER_DATA\"}" \
  --display-name "fn0-image-builder-temp" \
  --query 'data.id' \
  --raw-output)

cleanup() {
  oci compute instance terminate --instance-id "$INSTANCE_ID" --preserve-boot-volume false --force 2>/dev/null || true
}
trap cleanup EXIT

for i in $(seq 1 60); do
  STATE=$(oci compute instance get --instance-id "$INSTANCE_ID" --query 'data."lifecycle-state"' --raw-output 2>/dev/null || echo "")
  if [ "$STATE" = "RUNNING" ]; then
    break
  fi
  sleep 10
done

sleep 180

oci compute instance action --instance-id "$INSTANCE_ID" --action STOP --force 2>/dev/null
for i in $(seq 1 60); do
  STATE=$(oci compute instance get --instance-id "$INSTANCE_ID" --query 'data."lifecycle-state"' --raw-output 2>/dev/null || echo "")
  if [ "$STATE" = "STOPPED" ]; then
    break
  fi
  sleep 10
done

IMAGE_ID=$(oci compute image create \
  --compartment-id "$COMPARTMENT_ID" \
  --instance-id "$INSTANCE_ID" \
  --display-name "$CUSTOM_IMAGE_NAME" \
  --query 'data.id' \
  --raw-output)

for i in $(seq 1 120); do
  STATE=$(oci compute image get --image-id "$IMAGE_ID" --query 'data."lifecycle-state"' --raw-output 2>/dev/null || echo "")
  if [ "$STATE" = "AVAILABLE" ]; then
    break
  fi
  sleep 10
done

echo -n "$IMAGE_ID"
