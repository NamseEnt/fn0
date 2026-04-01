import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";
import * as docker from "@pulumi/docker";
import type { HqArgs } from "../hqArgs.schema";

export function deployHqApplication(
  parent: pulumi.Resource,
  args: {
    k8sProvider: k8s.Provider;
    hqImage: docker.Image;
    otlpEndpoint: pulumi.Output<string>;
    hqArgs: HqArgs;
    regionalSubnetId: pulumi.Input<string>;
  }
): {
  deployment: k8s.apps.v1.Deployment;
  service: k8s.core.v1.Service;
} {
  const { k8sProvider, hqImage, otlpEndpoint, hqArgs, regionalSubnetId } = args;
  const appLabels = { app: "hq" };

  const hqArgsSecret = new k8s.core.v1.Secret(
    "hq-args-secret",
    {
      metadata: { labels: appLabels },
      stringData: {
        "hq-args.json": pulumi.jsonStringify(hqArgs),
      },
    },
    { provider: k8sProvider, parent }
  );

  const configMountPath = "/etc/hq-config";
  const configFilePath = `${configMountPath}/hq-args.json`;

  const deployment = new k8s.apps.v1.Deployment(
    "hq-deployment",
    {
      metadata: { labels: appLabels },
      spec: {
        replicas: 1,
        strategy: { type: "Recreate" },
        selector: { matchLabels: appLabels },
        template: {
          metadata: {
            labels: appLabels,
          },
          spec: {
            hostNetwork: true,
            dnsPolicy: "ClusterFirstWithHostNet",
            containers: [
              {
                name: appLabels.app,
                image: hqImage.repoDigest,
                ports: [{ containerPort: 8080 }],
                livenessProbe: {
                  httpGet: {
                    path: "/health",
                    port: 8080,
                  },
                  initialDelaySeconds: 15,
                  periodSeconds: 2,
                  timeoutSeconds: 2,
                  failureThreshold: 3,
                },
                volumeMounts: [
                  {
                    name: "hq-args-vol",
                    mountPath: configMountPath,
                    readOnly: true,
                  },
                ],
                env: [
                  {
                    name: "OTLP_ENDPOINT",
                    value: otlpEndpoint,
                  },
                  {
                    name: "HQ_ARGS_PATH",
                    value: configFilePath,
                  },
                ],
              },
            ],
            volumes: [
              {
                name: "hq-args-vol",
                secret: {
                  secretName: hqArgsSecret.metadata.name,
                },
              },
            ],
          },
        },
      },
    },
    {
      provider: k8sProvider,
      parent,
      customTimeouts: { create: "3m", update: "3m" },
    }
  );

  const service = new k8s.core.v1.Service(
    "hq-service",
    {
      metadata: {
        labels: appLabels,
        annotations: {
          "oci.oraclecloud.com/load-balancer-type": "lb",
          "service.beta.kubernetes.io/oci-load-balancer-subnet1": regionalSubnetId,
        },
      },
      spec: {
        type: "LoadBalancer",
        selector: appLabels,
        ports: [
          {
            name: "http",
            port: 8080,
            targetPort: 8080,
            protocol: "TCP",
          },
        ],
      },
    },
    { provider: k8sProvider, parent, dependsOn: [deployment] }
  );

  return { deployment, service };
}
