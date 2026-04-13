import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";
import * as grafana from "@pulumiverse/grafana";

// Please make grafana stack manually first with slug.
export function hqGrafana(
  parent: pulumi.Resource,
  {
    slug,
    k8sProvider,
    suffix,
  }: {
    regionSlug: pulumi.Input<string>;
    slug: pulumi.Input<string>;
    k8sProvider: k8s.Provider;
    suffix: pulumi.Input<string>;
  }
): {
  otlpEndpoint: pulumi.Output<string>;
  workerOtlpEndpoint: pulumi.Output<string>;
  workerOtlpBasicAuth: pulumi.Output<string>;
  release: k8s.helm.v3.Release;
} {
  const stack = pulumi.all([slug]).apply(([slug]) =>
    grafana.cloud.getStack({
      slug,
    })
  );

  const config = new pulumi.Config("grafana");
  const password = config.require("cloudAccessPolicyToken");
  const releaseName = pulumi.interpolate`grafana-k8s-monitoring-${suffix}`;
  const clusterName = pulumi.interpolate`fn0-${suffix}`;
  const namespace = "monitoring";
  const destinationName = "grafana-cloud-metrics";

  const release = new k8s.helm.v3.Release(
    `grafana-k8s-monitoring`,
    {
      name: releaseName,
      chart: "k8s-monitoring",
      repositoryOpts: {
        repo: "https://grafana.github.io/helm-charts",
      },
      namespace,
      createNamespace: true,
      values: {
        cluster: {
          name: clusterName,
        },
        destinations: {
          [destinationName]: {
            type: "prometheus",
            url: stack.prometheusRemoteWriteEndpoint,
            auth: {
              type: "basic",
              username: stack.prometheusUserId.apply((id) => id.toString()),
              password,
            },
          },
          "grafana-cloud-logs": {
            type: "loki",
            url: stack.logsUrl.apply((x) => `${x}/loki/api/v1/push`),
            auth: {
              type: "basic",
              username: stack.logsUserId.apply((id) => id.toString()),
              password,
            },
          },
          "gc-otlp-endpoint": {
            type: "otlp",
            url: stack.otlpUrl.apply((x) => `${x}/otlp`),
            protocol: "http",
            auth: {
              type: "basic",
              username: stack.id.apply((id) => id.toString()),
              password,
            },
            metrics: {
              enabled: true,
            },
            logs: {
              enabled: true,
            },
            traces: {
              enabled: true,
            },
          },
        },
        telemetryServices: {
          "kube-state-metrics": {
            deploy: true,
          },
        },
        clusterMetrics: {
          enabled: true,
          collector: "alloy-receiver",
        },
        clusterEvents: {
          enabled: true,
          collector: "alloy-receiver",
        },
        podLogsViaLoki: {
          enabled: true,
          collector: "alloy-logs",
        },
        collectors: {
          "alloy-receiver": {
            alloy: {},
          },
          "alloy-logs": {
            alloy: {
              mounts: {
                varlog: true,
              },
            },
          },
        },
        applicationObservability: {
          enabled: true,
          collector: "alloy-receiver",
          receivers: {
            otlp: {
              grpc: {
                enabled: true,
                port: 4317,
              },
              http: {
                enabled: true,
                port: 4318,
              },
            },
            zipkin: {
              enabled: true,
              port: 9411,
            },
          },
        },
      },
    },
    { provider: k8sProvider, parent }
  );

  const workerOtlpEndpoint = stack.otlpUrl.apply(
    (url) => `${url}:443`
  );
  const workerOtlpBasicAuth = pulumi
    .all([stack.id, password])
    .apply(([id, pw]) =>
      Buffer.from(`${id}:${pw}`).toString("base64")
    );

  const serviceAccount = new grafana.cloud.StackServiceAccount(
    "dashboard-sa",
    {
      stackSlug: slug,
      name: "pulumi-dashboard-deployer",
      role: "Editor",
    },
    { parent }
  );

  const saToken = new grafana.cloud.StackServiceAccountToken(
    "dashboard-sa-token",
    {
      stackSlug: slug,
      serviceAccountId: serviceAccount.id,
      name: "pulumi-token",
    },
    { parent }
  );

  const grafanaInstanceProvider = new grafana.Provider(
    "grafana-instance",
    {
      url: stack.url,
      auth: saToken.key,
    },
    { parent }
  );

  const dashboardJson = require("./fn0-cloud-dashboard.json");
  new grafana.oss.Dashboard(
    "fn0-cloud-dashboard",
    {
      configJson: JSON.stringify(dashboardJson),
    },
    { provider: grafanaInstanceProvider, parent }
  );

  return {
    otlpEndpoint: pulumi.interpolate`http://${releaseName}-alloy-receiver.${namespace}:4317`,
    workerOtlpEndpoint,
    workerOtlpBasicAuth,
    release,
  };
}
