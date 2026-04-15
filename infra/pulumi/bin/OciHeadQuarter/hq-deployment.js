"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deployHqApplication = deployHqApplication;
const pulumi = require("@pulumi/pulumi");
const k8s = require("@pulumi/kubernetes");
function deployHqApplication(parent, args) {
    const { k8sProvider, hqImage, otlpEndpoint, workerOtlpEndpoint, workerOtlpBasicAuth, hqArgs, } = args;
    const appLabels = { app: "hq" };
    const hqArgsSecret = new k8s.core.v1.Secret("hq-args-secret", {
        metadata: { labels: appLabels },
        stringData: {
            "hq-args.json": pulumi.jsonStringify(hqArgs),
        },
    }, { provider: k8sProvider, parent });
    const configMountPath = "/etc/hq-config";
    const configFilePath = `${configMountPath}/hq-args.json`;
    const deployment = new k8s.apps.v1.Deployment("hq-deployment", {
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
                            image: hqImage.ref,
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
                                {
                                    name: "WORKER_OTLP_ENDPOINT",
                                    value: workerOtlpEndpoint,
                                },
                                {
                                    name: "WORKER_OTLP_BASIC_AUTH",
                                    value: workerOtlpBasicAuth,
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
    }, {
        provider: k8sProvider,
        parent,
        customTimeouts: { create: "3m", update: "3m" },
    });
    return { deployment };
}
//# sourceMappingURL=hq-deployment.js.map