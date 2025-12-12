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
    suffix: pulumi.Output<string>;
  }
): {
  otlpEndpoint: pulumi.Output<string>;
  release: k8s.helm.v3.Release;
} {
  const stack = pulumi.all([slug]).apply(([slug]) =>
    grafana.cloud.getStack({
      slug,
    })
  );

  // {
  //   "alertmanagerIpAllowListCname": "src-ips.alertmanager-prod-ap-northeast-0.grafana.net",
  //   "alertmanagerName": "fn0-alerts",
  //   "alertmanagerStatus": "active",
  //   "alertmanagerUrl": "https://alertmanager-prod-ap-northeast-0.grafana.net",
  //   "alertmanagerUserId": 1432425,
  //   "clusterSlug": "prod-ap-northeast-0",
  //   "deleteProtection": true,
  //   "description": "",
  //   "fleetManagementName": "fn0-agent-management",
  //   "fleetManagementStatus": "active",
  //   "fleetManagementUrl": "https://fleet-management-prod-019.grafana.net",
  //   "fleetManagementUserId": 1472362,
  //   "graphiteIpAllowListCname": "src-ips.prometheus-prod-49-prod-ap-northeast-0.grafana.net",
  //   "graphiteName": "fn0-graphite",
  //   "graphitePrivateConnectivityInfoPrivateDns": "gr-prod-49-cortex-gw.ap-northeast-1.vpce.grafana.net",
  //   "graphitePrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-051636cff520ef98c",
  //   "graphiteStatus": "active",
  //   "graphiteUrl": "https://graphite-prod-49-prod-ap-northeast-0.grafana.net",
  //   "graphiteUserId": 2868958,
  //   "id": "1472362",
  //   "influxUrl": "https://prometheus-prod-49-prod-ap-northeast-0.grafana.net",
  //   "labels": {},
  //   "logsIpAllowListCname": "src-ips.logs-prod-030.grafana.net",
  //   "logsName": "fn0-logs",
  //   "logsPrivateConnectivityInfoPrivateDns": "loki-prod-030-cortex-gw.ap-northeast-1.vpce.grafana.net",
  //   "logsPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-07fa66b627bea1a15",
  //   "logsStatus": "active",
  //   "logsUrl": "https://logs-prod-030.grafana.net",
  //   "logsUserId": 1430155,
  //   "name": "fn0.grafana.net",
  //   "oncallApiUrl": "https://oncall-prod-us-central-0.grafana.net/oncall",
  //   "orgId": 1619714,
  //   "orgName": "fn0",
  //   "orgSlug": "fn0",
  //   "otlpPrivateConnectivityInfoPrivateDns": "prod-ap-northeast-0-otlp-gateway.ap-northeast-1.vpce.grafana.net",
  //   "otlpPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-0e949280515221972",
  //   "otlpUrl": "https://otlp-gateway-prod-ap-northeast-0.grafana.net",
  //   "pdcApiPrivateConnectivityInfoPrivateDns": "pdc-grafana-datasources-api.ap-northeast-1.vpce.grafana.net",
  //   "pdcApiPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-06c6ff3ded7aa2b10",
  //   "pdcGatewayPrivateConnectivityInfoPrivateDns": "pdc-grafana-datasources.ap-northeast-1.vpce.grafana.net",
  //   "pdcGatewayPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-00b5f1832420b56cf",
  //   "profilesIpAllowListCname": "src-ips.profiles-prod-019.grafana.net",
  //   "profilesName": "fn0-profiles",
  //   "profilesPrivateConnectivityInfoPrivateDns": "profiles-prod-019-cortex-gw.ap-northeast-1.vpce.grafana.net",
  //   "profilesPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-02346a2bb84a2066c",
  //   "profilesStatus": "active",
  //   "profilesUrl": "https://profiles-prod-019.grafana.net",
  //   "profilesUserId": 1472362,
  //   "prometheusIpAllowListCname": "src-ips.prometheus-prod-49-prod-ap-northeast-0.grafana.net",
  //   "prometheusName": "fn0-prom",
  //   "prometheusPrivateConnectivityInfoPrivateDns": "mimir-prod-49-cortex-gw.ap-northeast-1.vpce.grafana.net",
  //   "prometheusPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-00d3545889713203e",
  //   "prometheusRemoteEndpoint": "https://prometheus-prod-49-prod-ap-northeast-0.grafana.net/api/prom",
  //   "prometheusRemoteWriteEndpoint": "https://prometheus-prod-49-prod-ap-northeast-0.grafana.net/api/prom/push",
  //   "prometheusStatus": "active",
  //   "prometheusUrl": "https://prometheus-prod-49-prod-ap-northeast-0.grafana.net",
  //   "prometheusUserId": 2868957,
  //   "regionSlug": "prod-ap-northeast-0",
  //   "slug": "fn0",
  //   "status": "active",
  //   "tracesIpAllowListCname": "src-ips.tempo-prod-20-prod-ap-northeast-0.grafana.net",
  //   "tracesName": "fn0-traces",
  //   "tracesPrivateConnectivityInfoPrivateDns": "tempo-prod-20-cortex-gw.ap-northeast-1.vpce.grafana.net",
  //   "tracesPrivateConnectivityInfoServiceName": "com.amazonaws.vpce.ap-northeast-1.vpce-svc-05adc09cbbe54b87d",
  //   "tracesStatus": "active",
  //   "tracesUrl": "https://tempo-prod-20-prod-ap-northeast-0.grafana.net",
  //   "tracesUserId": 1424465,
  //   "url": "https://fn0.grafana.net"
  // }

  const config = new pulumi.Config("grafana");
  const password = config.require("cloudAccessPolicyToken");
  const releaseName = pulumi.interpolate`grafana-k8s-monitoring-${suffix}`;
  const clusterName = pulumi.interpolate`fn0-${suffix}`;
  const namespace = "monitoring";
  const destinationName = "grafana-cloud-metrics";
  const secretName = pulumi.interpolate`${destinationName}-${releaseName}`;

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
        destinations: [
          {
            name: destinationName,
            type: "prometheus",
            url: stack.prometheusRemoteWriteEndpoint,
            auth: {
              type: "basic",
              username: stack.prometheusUserId.apply((id) => id.toString()),
              password,
            },
          },
          {
            name: "grafana-cloud-logs",
            type: "loki",
            url: stack.logsUrl.apply((x) => `${x}/loki/api/v1/push`),
            auth: {
              type: "basic",
              username: stack.logsUserId.apply((id) => id.toString()),
              password,
            },
          },
          {
            name: "gc-otlp-endpoint",
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
        ],
        clusterMetrics: {
          enabled: true,
          opencost: {
            enabled: true,
            metricsSource: destinationName,
            opencost: {
              exporter: {
                defaultClusterId: clusterName,
              },
              prometheus: {
                existingSecretName: secretName,
                external: {
                  url: stack.prometheusRemoteEndpoint,
                },
              },
            },
          },
          kepler: {
            enabled: true,
          },
        },
        clusterEvents: {
          enabled: true,
        },
        podLogs: {
          enabled: true,
        },
        applicationObservability: {
          enabled: true,
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
        "alloy-metrics": {
          enabled: true,
          alloy: {},
          remoteConfig: {
            enabled: false,
          },
        },
        "alloy-singleton": {
          enabled: true,
          alloy: {},
          remoteConfig: {
            enabled: false,
          },
        },
        "alloy-logs": {
          enabled: true,
          alloy: {},
          remoteConfig: {
            enabled: false,
          },
        },
        "alloy-receiver": {
          enabled: true,
          alloy: {},
          remoteConfig: {
            enabled: false,
          },
        },
      },
    },
    { provider: k8sProvider, parent }
  );

  return {
    otlpEndpoint: pulumi.interpolate`http://${releaseName}-alloy-receiver.${namespace}:4317`,
    release,
  };
}
