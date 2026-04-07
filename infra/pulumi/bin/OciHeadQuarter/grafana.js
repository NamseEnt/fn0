"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hqGrafana = hqGrafana;
const pulumi = require("@pulumi/pulumi");
const k8s = require("@pulumi/kubernetes");
const grafana = require("@pulumiverse/grafana");
// Please make grafana stack manually first with slug.
function hqGrafana(parent, { slug, k8sProvider, suffix, }) {
    const stack = pulumi.all([slug]).apply(([slug]) => grafana.cloud.getStack({
        slug,
    }));
    const config = new pulumi.Config("grafana");
    const password = config.require("cloudAccessPolicyToken");
    const releaseName = pulumi.interpolate `grafana-k8s-monitoring-${suffix}`;
    const clusterName = pulumi.interpolate `fn0-${suffix}`;
    const namespace = "monitoring";
    const destinationName = "grafana-cloud-metrics";
    const secretName = pulumi.interpolate `${destinationName}-${releaseName}`;
    const release = new k8s.helm.v3.Release(`grafana-k8s-monitoring`, {
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
    }, { provider: k8sProvider, parent });
    return {
        otlpEndpoint: pulumi.interpolate `http://${releaseName}-alloy-receiver.${namespace}:4317`,
        release,
    };
}
//# sourceMappingURL=grafana.js.map