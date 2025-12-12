import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";

export function deployK8sDashboard(
  parent: pulumi.Resource,
  {
    k8sProvider,
    dependsOn,
  }: {
    k8sProvider: k8s.Provider;
    dependsOn?: pulumi.Input<pulumi.Resource>[];
  }
) {
  const namespace = new k8s.core.v1.Namespace(
    "k8s-dashboard-namespace",
    {
      metadata: {
        name: "kubernetes-dashboard",
      },
    },
    { provider: k8sProvider, parent, dependsOn }
  );

  // Deploy Kubernetes Dashboard via Helm
  const dashboardRelease = new k8s.helm.v3.Release(
    "kubernetes-dashboard",
    {
      name: "kubernetes-dashboard",
      chart: "kubernetes-dashboard",
      repositoryOpts: {
        repo: "https://kubernetes.github.io/dashboard/",
      },
      namespace: "kubernetes-dashboard",
      createNamespace: true,
      values: {
        kong: {
          image: {
            repository: "docker.io/library/kong",
            tag: "3.9.0",
          },
          proxy: {
            http: {
              enabled: true,
            },
          },
        },
        app: {
          ingress: {
            enabled: false,
          },
        },
      },
    },
    { provider: k8sProvider, parent }
  );

  // Create admin ServiceAccount for full access
  const adminServiceAccount = new k8s.core.v1.ServiceAccount(
    "dashboard-admin-sa",
    {
      metadata: {
        name: "dashboard-admin",
        namespace: namespace.metadata.name,
      },
    },
    { provider: k8sProvider, parent, dependsOn: [dashboardRelease] }
  );

  // Create ClusterRoleBinding for admin access
  const adminClusterRoleBinding = new k8s.rbac.v1.ClusterRoleBinding(
    "dashboard-admin-crb",
    {
      metadata: {
        name: "dashboard-admin",
      },
      roleRef: {
        apiGroup: "rbac.authorization.k8s.io",
        kind: "ClusterRole",
        name: "cluster-admin",
      },
      subjects: [
        {
          kind: "ServiceAccount",
          name: adminServiceAccount.metadata.name,
          namespace: namespace.metadata.name,
        },
      ],
    },
    { provider: k8sProvider, parent, dependsOn: [adminServiceAccount] }
  );

  // Create a Secret for the ServiceAccount token (for K8s 1.24+)
  const adminToken = new k8s.core.v1.Secret(
    "dashboard-admin-token",
    {
      metadata: {
        name: "dashboard-admin-token",
        namespace: namespace.metadata.name,
        annotations: {
          "kubernetes.io/service-account.name":
            adminServiceAccount.metadata.name,
        },
      },
      type: "kubernetes.io/service-account-token",
    },
    { provider: k8sProvider, parent, dependsOn: [adminServiceAccount] }
  );

  return {
    namespace,
    dashboardRelease,
    adminServiceAccount,
    adminClusterRoleBinding,
    adminToken,
  };
}
