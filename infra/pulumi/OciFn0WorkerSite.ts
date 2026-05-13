import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as tls from "@pulumi/tls";
import * as random from "@pulumi/random";
import { CustomWorkerImage } from "./CustomWorkerImage";

export interface OciFn0WorkerSiteArgs {
  region: pulumi.Input<string>;
  count: number;
  shape: pulumi.Input<string>;
  ocpus: pulumi.Input<number>;
  memoryInGbs: pulumi.Input<number>;
  workerAgentVersion: string;
  agentDocDb: AgentDocDbArgs;
  agentDnsRegister: AgentDnsRegisterArgs;
  worker: WorkerArgs;
}

export interface AgentDocDbArgs {
  url: pulumi.Input<string>;
  authToken: pulumi.Input<string>;
}

export interface AgentDnsRegisterArgs {
  apiToken: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  // Comma-separated list of A-record hostnames the agent registers /
  // deregisters on its public IP. Must include the SaaS fallback origin
  // hostname (e.g. `fallback.${domain}`) because Cloudflare requires an
  // explicit proxied A record for the fallback origin, not just a wildcard.
  hostnames: pulumi.Input<string>;
}

export interface WorkerArgs {
  tlsOrigin: WorkerTlsOriginArgs;
  envEncryptionKeyBase64: pulumi.Input<string>;
  forteDb: WorkerForteDbArgs;
  controlInvokeAllowedSubdomain: pulumi.Input<string>;
  vault: WorkerVaultArgs;
  otlp: WorkerOtlpArgs;
  hostObservability: WorkerHostObservabilityArgs;
  bundleStorage: WorkerBundleStorageArgs;
}

export interface WorkerBundleStorageArgs {
  bucketName: pulumi.Input<string>;
  endpoint: pulumi.Input<string>;
  region: pulumi.Input<string>;
  accessKeyId: pulumi.Input<string>;
  secretAccessKey: pulumi.Input<string>;
}

export interface WorkerOtlpArgs {
  endpoint: pulumi.Input<string>;
  basicAuth: pulumi.Input<string>;
}

export interface WorkerHostObservabilityArgs {
  prometheusUrl: pulumi.Input<string>;
  prometheusUserId: pulumi.Input<string>;
  lokiUrl: pulumi.Input<string>;
  lokiUserId: pulumi.Input<string>;
  basicAuthPassword: pulumi.Input<string>;
}

export interface WorkerTlsOriginArgs {
  certPem: pulumi.Input<string>;
  keyPem: pulumi.Input<string>;
}

export interface WorkerForteDbArgs {
  groupToken: pulumi.Input<string>;
  hostSuffix: pulumi.Input<string>;
}

export interface WorkerVaultArgs {
  cryptoEndpoint: pulumi.Input<string>;
  keyOcid: pulumi.Input<string>;
  region: pulumi.Input<string>;
  allowedSubdomain: pulumi.Input<string>;
  workerCredentials: pulumi.Input<WorkerVaultCredentials>;
}

export interface WorkerVaultCredentials {
  ociUserId: string;
  ociTenancyId: string;
  ociFingerprint: string;
  ociPrivateKeyBase64: string;
}

export interface OciCwasmBucketInfo {
  endpoint: pulumi.Output<string>;
  region: pulumi.Output<string>;
  bucketName: pulumi.Output<string>;
  accessKeyId: pulumi.Output<string>;
  secretAccessKey: pulumi.Output<string>;
  namespace: pulumi.Output<string>;
}

export interface OciQueueInfo {
  ocid: pulumi.Output<string>;
  messagesEndpoint: pulumi.Output<string>;
  region: pulumi.Output<string>;
  ociUserId: pulumi.Output<string>;
  ociTenancyId: pulumi.Output<string>;
  ociFingerprint: pulumi.Output<string>;
  ociPrivateKeyBase64: pulumi.Output<string>;
}

export interface WorkerImageRegistry {
  url: string;
  username: string;
  password: string;
  repository: string;
}

export interface OciFn0WorkerSiteInfraEnvs {
  OCI_PRIVATE_KEY_BASE64: string;
  OCI_USER_ID: string;
  OCI_FINGERPRINT: string;
  OCI_TENANCY_ID: string;
  OCI_REGION: string;
  OCI_COMPARTMENT_ID: string;
  OCI_INSTANCE_CONFIGURATION_ID: string;
  OCI_AVAILABILITY_DOMAIN: string;
}

const MANAGED_BY_TAG_VALUE = "fn0-control";

export class OciFn0WorkerSite extends pulumi.ComponentResource {
  public readonly compartmentId: pulumi.Output<string>;
  public readonly subnetId: pulumi.Output<string>;
  public readonly instanceConfigurationId: pulumi.Output<string>;
  public readonly infraEnvs: pulumi.Output<OciFn0WorkerSiteInfraEnvs>;
  public readonly workerImageRegistries: pulumi.Output<WorkerImageRegistry[]>;
  public readonly osImageId: pulumi.Output<string>;
  public readonly cwasmBucket: OciCwasmBucketInfo;
  public readonly queue: OciQueueInfo;
  public readonly sshPublicKey: pulumi.Output<string>;
  public readonly sshPrivateKey: pulumi.Output<string>;
  public readonly instanceIds: pulumi.Output<string[]>;
  public readonly publicIps: pulumi.Output<(string | undefined)[]>;

  constructor(
    name: string,
    args: OciFn0WorkerSiteArgs,
    opts?: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:oci-fn0-worker-site", name, args, opts);

    const compartmentSuffix = new random.RandomString(
      "compartment-suffix",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    ).result;

    const compartment = new oci.identity.Compartment(
      "compartment",
      {
        description: "Compartment for fn0 OCI Worker",
        name: pulumi.interpolate`fn0-host-${compartmentSuffix}`,
        enableDelete: true,
      },
      { parent: this }
    );

    this.compartmentId = compartment.id;

    const privateKey = new tls.PrivateKey(
      "oci-api-key-pair",
      {
        algorithm: "RSA",
        rsaBits: 2048,
      },
      { parent: this }
    );

    const workerManager = new oci.identity.User(
      "worker-manager",
      {
        description: "fn0 worker manager",
      },
      { parent: this }
    );

    const apiKey = new oci.identity.ApiKey(
      "worker-api-key",
      {
        userId: workerManager.id,
        keyValue: privateKey.publicKeyPem,
      },
      { parent: this }
    );

    const group = new oci.identity.Group(
      "worker-manager-group",
      {
        description: "fn0 worker manager group",
      },
      { parent: this }
    );

    new oci.identity.UserGroupMembership(
      "worker-manager-group-membership",
      {
        userId: workerManager.id,
        groupId: group.id,
      },
      { parent: this }
    );

    new oci.identity.Policy(
      "worker-manager-policy",
      {
        compartmentId: workerManager.compartmentId,
        description: "Policy for fn0 worker manager",
        statements: [
          pulumi.interpolate`Allow group ${group.name} to manage instance-family in compartment id ${compartment.id}`,
          pulumi.interpolate`Allow group ${group.name} to manage instance-configurations in compartment id ${compartment.id}`,
          pulumi.interpolate`Allow group ${group.name} to manage compute-container-family in compartment id ${compartment.id}`,
          pulumi.interpolate`Allow group ${group.name} to use virtual-network-family in compartment id ${compartment.id}`,
          pulumi.interpolate`Allow group ${group.name} to read app-catalog-listing in compartment id ${compartment.id}`,
          pulumi.interpolate`Allow group ${group.name} to use tag-namespaces in tenancy`,
        ],
      },
      { parent: this }
    );

    const imageBuilderDynGroup = new oci.identity.DynamicGroup(
      "image-builder-dyn-group",
      {
        compartmentId: workerManager.compartmentId,
        description: "Instances that can self-terminate for image building",
        matchingRule: pulumi.interpolate`ANY {instance.compartment.id = '${compartment.id}'}`,
        name: pulumi.interpolate`fn0-image-builder-${compartmentSuffix}`,
      },
      { parent: this }
    );

    new oci.identity.Policy(
      "image-builder-self-terminate-policy",
      {
        compartmentId: workerManager.compartmentId,
        description: "Allow image builder instances to terminate themselves",
        statements: [
          pulumi.interpolate`Allow dynamic-group ${imageBuilderDynGroup.name} to manage instance-family in compartment id ${compartment.id}`,
        ],
      },
      { parent: this, dependsOn: [imageBuilderDynGroup] }
    );

    const vcn = new oci.core.Vcn(
      "vcn",
      {
        compartmentId: compartment.id,
        isIpv6enabled: true,
        isOracleGuaAllocationEnabled: true,
        cidrBlocks: ["10.0.0.0/16"],
      },
      { parent: this }
    );

    const workerSshKey = new tls.PrivateKey(
      "worker-ssh-key",
      {
        algorithm: "RSA",
        rsaBits: 4096,
      },
      { parent: this }
    );

    this.sshPublicKey = workerSshKey.publicKeyOpenssh;
    this.sshPrivateKey = workerSshKey.privateKeyPem;

    const securityList = new oci.core.SecurityList(
      "security-list",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        ingressSecurityRules: [
          {
            protocol: "6",
            source: "0.0.0.0/0",
            tcpOptions: { min: 22, max: 22 },
          },
          {
            protocol: "6",
            source: "::/0",
            tcpOptions: { min: 22, max: 22 },
          },
          ...[...cloudflareIpv4Ranges, ...cloudflareIpv6Ranges].map(
            (source) => ({
              protocol: "6",
              source,
              tcpOptions: { min: 443, max: 443 },
            })
          ),
        ],
        egressSecurityRules: [
          {
            destination: "0.0.0.0/0",
            protocol: "all",
          },
          {
            destination: "::/0",
            protocol: "all",
          },
        ],
      },
      { parent: this }
    );

    const internetGateway = new oci.core.InternetGateway(
      "igw",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
      },
      { parent: this }
    );

    const routeTable = new oci.core.RouteTable(
      "route-table",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        routeRules: [
          {
            destination: "::/0",
            destinationType: "CIDR_BLOCK",
            networkEntityId: internetGateway.id,
          },
          {
            destination: "0.0.0.0/0",
            destinationType: "CIDR_BLOCK",
            networkEntityId: internetGateway.id,
          },
        ],
      },
      { parent: this }
    );

    const subnet = new oci.core.Subnet(
      "subnet",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        ipv4cidrBlocks: ["10.0.0.0/24"],
        ipv6cidrBlocks: vcn.ipv6cidrBlocks.apply((x) =>
          x.map((x) => x.replace("/56", "/64"))
        ),
        prohibitInternetIngress: false,
        prohibitPublicIpOnVnic: false,
        securityListIds: [securityList.id],
        routeTableId: routeTable.id,
      },
      { parent: this }
    );

    const availabilityDomain = compartment.id.apply((compartmentId) =>
      oci.identity
        .getAvailabilityDomains({
          compartmentId,
        })
        .then((x) => {
          const ad = x.availabilityDomains[0]?.name;
          if (!ad) {
            throw new Error("can not find availability domain");
          }
          return ad;
        })
    );

    const baseImageId = compartment.id.apply((compartmentId) =>
      oci.core
        .getImages({
          compartmentId,
          operatingSystem: "Oracle Linux",
          operatingSystemVersion: "10",
          sortOrder: "DESC",
        })
        .then((x) => {
          const imageId = x.images.find(
            (x) => x.createImageAllowed && x.displayName.includes("-aarch64-")
          )?.id;

          if (!imageId) {
            throw new Error("can not find image");
          }

          return imageId;
        })
    );

    const customWorkerImage = new CustomWorkerImage(
      "custom-worker-image",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        availabilityDomain,
        baseImageId,
        displayName: "fn0-ol10-podman-aarch64",
        internetGatewayId: internetGateway.id,
      },
      { parent: this }
    );

    const imageId = customWorkerImage.imageId;

    const instanceConfiguration = new oci.core.InstanceConfiguration(
      "instance-configuration",
      {
        compartmentId: compartment.id,
        instanceDetails: {
          instanceType: "compute",
          launchDetails: {
            shape: args.shape,
            shapeConfig: {
              ocpus: args.ocpus,
              memoryInGbs: args.memoryInGbs,
            },
            sourceDetails: {
              sourceType: "image",
              imageId,
            },
            createVnicDetails: {
              subnetId: subnet.id,
              assignIpv6ip: true,
              assignPublicIp: true,
            },
          },
        },
      },
      { parent: this }
    );

    this.subnetId = subnet.id;
    this.instanceConfigurationId = instanceConfiguration.id;

    this.infraEnvs = pulumi
      .all([
        privateKey.privateKeyPemPkcs8,
        workerManager.id,
        workerManager.compartmentId,
        compartment.id,
        instanceConfiguration.id,
        apiKey.fingerprint,
        pulumi.output(availabilityDomain),
        pulumi.output(args.region),
      ])
      .apply(
        ([
          privateKeyPem,
          userId,
          tenancyId,
          compartmentId,
          instanceConfigurationId,
          fingerprint,
          availabilityDomain,
          region,
        ]) => ({
          OCI_PRIVATE_KEY_BASE64: Buffer.from(privateKeyPem).toString("base64"),
          OCI_USER_ID: userId,
          OCI_FINGERPRINT: fingerprint,
          OCI_TENANCY_ID: tenancyId,
          OCI_REGION: region,
          OCI_COMPARTMENT_ID: compartmentId,
          OCI_INSTANCE_CONFIGURATION_ID: instanceConfigurationId,
          OCI_AVAILABILITY_DOMAIN: availabilityDomain,
        })
      );

    const workerRepo = new oci.artifacts.ContainerRepository(
      "worker-repo",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-worker-${compartmentSuffix}`,
        isPublic: true,
      },
      { parent: this, retainOnDelete: false }
    );

    new oci.artifacts.ContainerRepository(
      "worker-agent-repo",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-worker-${compartmentSuffix}-agent`,
        isPublic: true,
      },
      { parent: this, retainOnDelete: false }
    );

    const workerDockerUser = new oci.identity.User(
      "worker-docker-user",
      {
        name: pulumi.interpolate`fn0-worker-docker-${compartmentSuffix}`,
        description: "User for fn0-worker image push",
      },
      { parent: this }
    );

    const workerDockerGroup = new oci.identity.Group(
      "worker-docker-group",
      {
        name: pulumi.interpolate`fn0-worker-pushers-${compartmentSuffix}`,
        description: "Group allowed to push worker images",
      },
      { parent: this }
    );

    new oci.identity.UserGroupMembership(
      "worker-docker-membership",
      {
        userId: workerDockerUser.id,
        groupId: workerDockerGroup.id,
      },
      { parent: this }
    );

    new oci.identity.Policy(
      "worker-ocir-push-policy",
      {
        compartmentId: compartment.id,
        name: pulumi.interpolate`allow-worker-push-${compartmentSuffix}`,
        description: "Policy to allow worker image push",
        statements: [
          pulumi.interpolate`Allow group ${workerDockerGroup.name} to manage repos in compartment id ${compartment.id}`,
        ],
      },
      { dependsOn: [workerDockerGroup], parent: this }
    );

    const workerAuthToken = new oci.identity.AuthToken(
      "worker-auth-token",
      {
        userId: workerDockerUser.id,
        description: "AuthToken for fn0-worker image push",
      },
      { parent: this }
    );

    const registryUrl = pulumi.interpolate`ocir.${args.region}.oci.oraclecloud.com`;

    this.workerImageRegistries = pulumi
      .all([
        registryUrl,
        workerRepo.namespace,
        workerRepo.displayName,
        workerDockerUser.name,
        workerAuthToken.token,
      ])
      .apply(([url, namespace, repoName, userName, token]) => [
        {
          url,
          username: `${namespace}/${userName}`,
          password: token,
          repository: `${namespace}/${repoName}`,
        },
      ]);
    this.osImageId = imageId;

    const cwasmBucketUser = new oci.identity.User(
      "cwasm-bucket-user",
      {
        name: pulumi.interpolate`fn0-cwasm-${compartmentSuffix}`,
        description: "User for cwasm S3-compatible access",
      },
      { parent: this }
    );

    const cwasmBucketGroup = new oci.identity.Group(
      "cwasm-bucket-group",
      {
        name: pulumi.interpolate`fn0-cwasm-group-${compartmentSuffix}`,
        description: "Group for cwasm bucket access",
      },
      { parent: this }
    );

    new oci.identity.UserGroupMembership(
      "cwasm-bucket-membership",
      {
        userId: cwasmBucketUser.id,
        groupId: cwasmBucketGroup.id,
      },
      { parent: this }
    );

    new oci.identity.Policy(
      "cwasm-bucket-policy",
      {
        compartmentId: compartment.id,
        name: pulumi.interpolate`allow-cwasm-access-${compartmentSuffix}`,
        description: "Policy for cwasm bucket access",
        statements: [
          pulumi.interpolate`Allow group ${cwasmBucketGroup.name} to manage objects in compartment id ${compartment.id}`,
          pulumi.interpolate`Allow group ${cwasmBucketGroup.name} to manage buckets in compartment id ${compartment.id}`,
        ],
      },
      { dependsOn: [cwasmBucketGroup], parent: this }
    );

    const customerSecretKey = new oci.identity.CustomerSecretKey(
      "cwasm-s3-key",
      {
        userId: cwasmBucketUser.id,
        displayName: "cwasm-s3-compatible-key",
      },
      { parent: this }
    );

    const ociObjectStorageBucket = new oci.objectstorage.Bucket(
      "cwasm-bucket",
      {
        compartmentId: compartment.id,
        name: pulumi.interpolate`fn0-cwasm-${compartmentSuffix}`,
        namespace: workerRepo.namespace,
        accessType: "NoPublicAccess",
      },
      { parent: this }
    );

    this.cwasmBucket = {
      endpoint: pulumi.interpolate`https://${workerRepo.namespace}.compat.objectstorage.${args.region}.oraclecloud.com`,
      region: pulumi.output(args.region),
      bucketName: ociObjectStorageBucket.name,
      accessKeyId: customerSecretKey.id,
      secretAccessKey: customerSecretKey.key,
      namespace: workerRepo.namespace,
    };

    const queue = new oci.queue.Queue(
      "queue",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-queue-${compartmentSuffix}`,
      },
      { parent: this }
    );

    const queueUser = new oci.identity.User(
      "queue-user",
      {
        name: pulumi.interpolate`fn0-queue-${compartmentSuffix}`,
        description: "User for fn0 queue produce/consume",
      },
      { parent: this }
    );

    const queueGroup = new oci.identity.Group(
      "queue-group",
      {
        name: pulumi.interpolate`fn0-queue-group-${compartmentSuffix}`,
        description: "Group for fn0 queue access",
      },
      { parent: this }
    );

    new oci.identity.UserGroupMembership(
      "queue-membership",
      {
        userId: queueUser.id,
        groupId: queueGroup.id,
      },
      { parent: this }
    );

    new oci.identity.Policy(
      "queue-policy",
      {
        compartmentId: compartment.id,
        name: pulumi.interpolate`allow-queue-${compartmentSuffix}`,
        description: "Policy for fn0 queue produce/consume",
        statements: [
          pulumi.interpolate`Allow group ${queueGroup.name} to use queues in compartment id ${compartment.id}`,
        ],
      },
      { dependsOn: [queueGroup], parent: this }
    );

    const queueApiPrivateKey = new tls.PrivateKey(
      "queue-api-key-pair",
      {
        algorithm: "RSA",
        rsaBits: 2048,
      },
      { parent: this }
    );

    const queueApiKey = new oci.identity.ApiKey(
      "queue-api-key",
      {
        userId: queueUser.id,
        keyValue: queueApiPrivateKey.publicKeyPem,
      },
      { parent: this }
    );

    this.queue = {
      ocid: queue.id,
      messagesEndpoint: queue.messagesEndpoint,
      region: pulumi.output(args.region),
      ociUserId: queueUser.id,
      ociTenancyId: queueUser.compartmentId,
      ociFingerprint: queueApiKey.fingerprint,
      ociPrivateKeyBase64: queueApiPrivateKey.privateKeyPemPkcs8.apply((pem) =>
        Buffer.from(pem).toString("base64")
      ),
    };

    const agentImageRef = this.workerImageRegistries.apply((regs) => {
      if (regs.length === 0) {
        throw new Error("workerImageRegistries is empty");
      }
      const r = regs[0];
      return `${r.url}/${r.repository}-agent:${args.workerAgentVersion}`;
    });

    const agentEnv = buildAgentEnv(args.agentDocDb, args.agentDnsRegister);
    const workerEnv = buildWorkerEnv(args.worker, this.cwasmBucket, this.queue);
    const alloyConfig = pulumi
      .all([
        args.worker.hostObservability.prometheusUrl,
        args.worker.hostObservability.prometheusUserId,
        args.worker.hostObservability.lokiUrl,
        args.worker.hostObservability.lokiUserId,
        args.worker.hostObservability.basicAuthPassword,
      ])
      .apply(([promUrl, promUser, lokiUrl, lokiUser, password]) =>
        renderAlloyConfig({ promUrl, promUser, lokiUrl, lokiUser, password })
      );

    const cloudInit = pulumi
      .all([agentImageRef, agentEnv, workerEnv, alloyConfig])
      .apply(([agentImageRef, agentEnv, workerEnv, alloyConfig]) =>
        renderCloudInit(agentImageRef, agentEnv, workerEnv, alloyConfig)
      );
    const userData = cloudInit.apply((s) =>
      Buffer.from(s, "utf8").toString("base64")
    );

    const metadata = pulumi
      .all([userData, this.sshPublicKey])
      .apply(([ud, ssh]) => {
        const m: { [k: string]: string } = { user_data: ud };
        if (ssh) m["ssh_authorized_keys"] = ssh;
        return m;
      });

    const instances: oci.core.Instance[] = [];
    for (let i = 0; i < args.count; i++) {
      instances.push(
        new oci.core.Instance(
          `instance-${i}`,
          {
            compartmentId: compartment.id,
            availabilityDomain,
            displayName: `${name}-${i}`,
            shape: args.shape,
            shapeConfig: {
              ocpus: args.ocpus,
              memoryInGbs: args.memoryInGbs,
            },
            sourceDetails: {
              sourceType: "image",
              sourceId: imageId,
            },
            createVnicDetails: {
              subnetId: subnet.id,
              assignPublicIp: "true",
              assignIpv6ip: true,
            },
            metadata,
            freeformTags: {
              managed_by: MANAGED_BY_TAG_VALUE,
              fn0_role: "worker",
            },
          },
          { parent: this }
        )
      );
    }

    this.instanceIds = pulumi.all(instances.map((i) => i.id));
    this.publicIps = pulumi.all(instances.map((i) => i.publicIp));

    this.registerOutputs({
      compartmentId: this.compartmentId,
      subnetId: this.subnetId,
      instanceConfigurationId: this.instanceConfigurationId,
      infraEnvs: this.infraEnvs,
      workerImageRegistries: this.workerImageRegistries,
      osImageId: this.osImageId,
      cwasmBucket: this.cwasmBucket,
      queue: this.queue,
      sshPublicKey: this.sshPublicKey,
      sshPrivateKey: this.sshPrivateKey,
      instanceIds: this.instanceIds,
      publicIps: this.publicIps,
    });
  }
}

function buildAgentEnv(
  docDb: AgentDocDbArgs,
  dnsRegister: AgentDnsRegisterArgs
): pulumi.Output<{ [k: string]: string }> {
  const base: { [k: string]: pulumi.Input<string> } = {
    TURSO_URL: docDb.url,
    TURSO_AUTH_TOKEN: docDb.authToken,
    FN0_AGENT_DNS_API_TOKEN: dnsRegister.apiToken,
    FN0_AGENT_DNS_ZONE_ID: dnsRegister.zoneId,
    FN0_AGENT_DNS_HOSTNAMES: dnsRegister.hostnames,
  };
  return resolveEnvMap(base);
}

function buildWorkerEnv(
  worker: WorkerArgs,
  cwasmBucket: OciCwasmBucketInfo,
  queue: OciQueueInfo
): pulumi.Output<{ [k: string]: string }> {
  const otlpParsed = pulumi.output(worker.otlp.endpoint).apply((u) => {
    const url = new URL(u);
    return {
      targetHost: url.host,
      targetPathPrefix: url.pathname.replace(/\/$/, ""),
    };
  });

  const vaultCredsEnv = pulumi
    .output(worker.vault.workerCredentials)
    .apply((c): { [k: string]: string } => ({
      FN0_VAULT_OCI_USER_ID: c.ociUserId,
      FN0_VAULT_OCI_TENANCY_ID: c.ociTenancyId,
      FN0_VAULT_OCI_FINGERPRINT: c.ociFingerprint,
      FN0_VAULT_OCI_PRIVATE_KEY_BASE64: c.ociPrivateKeyBase64,
    }));

  const base: { [k: string]: pulumi.Input<string> } = {
    CWASM_BUCKET: worker.bundleStorage.bucketName,
    S3_ENDPOINT: worker.bundleStorage.endpoint,
    S3_REGION: worker.bundleStorage.region,
    AWS_ACCESS_KEY_ID: worker.bundleStorage.accessKeyId,
    AWS_SECRET_ACCESS_KEY: worker.bundleStorage.secretAccessKey,

    ORIGIN_CERT_PEM_BASE64: pulumi
      .output(worker.tlsOrigin.certPem)
      .apply((s) => Buffer.from(s, "utf8").toString("base64")),
    ORIGIN_KEY_PEM_BASE64: pulumi
      .output(worker.tlsOrigin.keyPem)
      .apply((s) => Buffer.from(s, "utf8").toString("base64")),

    FN0_ENV_KEY_BASE64: worker.envEncryptionKeyBase64,

    TURSO_GROUP_TOKEN: worker.forteDb.groupToken,
    TURSO_DB_HOST_SUFFIX: worker.forteDb.hostSuffix,

    FN0_QUEUE_OCID: queue.ocid,
    FN0_QUEUE_MESSAGES_ENDPOINT: queue.messagesEndpoint,
    FN0_QUEUE_REGION: queue.region,
    FN0_QUEUE_OCI_USER_ID: queue.ociUserId,
    FN0_QUEUE_OCI_TENANCY_ID: queue.ociTenancyId,
    FN0_QUEUE_OCI_FINGERPRINT: queue.ociFingerprint,
    FN0_QUEUE_OCI_PRIVATE_KEY_BASE64: queue.ociPrivateKeyBase64,

    FN0_CONTROL_INVOKE_QUEUE_OCID: queue.ocid,
    FN0_CONTROL_INVOKE_QUEUE_MESSAGES_ENDPOINT: queue.messagesEndpoint,
    FN0_CONTROL_INVOKE_QUEUE_OCI_USER_ID: queue.ociUserId,
    FN0_CONTROL_INVOKE_QUEUE_OCI_TENANCY_ID: queue.ociTenancyId,
    FN0_CONTROL_INVOKE_QUEUE_OCI_FINGERPRINT: queue.ociFingerprint,
    FN0_CONTROL_INVOKE_QUEUE_OCI_PRIVATE_KEY_BASE64: queue.ociPrivateKeyBase64,
    FN0_CONTROL_INVOKE_QUEUE_ALLOWED_SUBDOMAIN:
      worker.controlInvokeAllowedSubdomain,

    FN0_VAULT_CRYPTO_ENDPOINT: worker.vault.cryptoEndpoint,
    FN0_VAULT_KEY_OCID: worker.vault.keyOcid,
    FN0_VAULT_REGION: worker.vault.region,
    FN0_VAULT_ALLOWED_SUBDOMAIN: worker.vault.allowedSubdomain,

    OTLP_ENDPOINT: worker.otlp.endpoint,
    OTLP_BASIC_AUTH: worker.otlp.basicAuth,
    FN0_OTLP_TARGET_HOST: otlpParsed.targetHost,
    FN0_OTLP_TARGET_PATH_PREFIX: otlpParsed.targetPathPrefix,
    FN0_OTLP_AUTH: worker.otlp.basicAuth,
  };

  return pulumi
    .all([resolveEnvMap(base), vaultCredsEnv])
    .apply(([b, v]) => ({ ...b, ...v }));
}

function resolveEnvMap(m: {
  [k: string]: pulumi.Input<string>;
}): pulumi.Output<{ [k: string]: string }> {
  const keys = Object.keys(m);
  const values = keys.map((k) => pulumi.output(m[k]));
  return pulumi.all(values).apply((resolved) => {
    const out: { [k: string]: string } = {};
    keys.forEach((k, i) => {
      out[k] = resolved[i];
    });
    return out;
  });
}

const ALLOY_IMAGE_REF = "docker.io/grafana/alloy:latest";

function renderAlloyConfig(args: {
  promUrl: string;
  promUser: string;
  lokiUrl: string;
  lokiUser: string;
  password: string;
}): string {
  return `prometheus.exporter.unix "node" {
  set_collectors = ["cpu", "diskstats", "filesystem", "loadavg", "meminfo", "netdev", "netstat", "vmstat", "uname", "time"]
  rootfs_path     = "/host/root"
  procfs_path     = "/host/proc"
  sysfs_path      = "/host/sys"
}

prometheus.scrape "node" {
  targets    = prometheus.exporter.unix.node.targets
  forward_to = [prometheus.remote_write.default.receiver]
}

prometheus.remote_write "default" {
  endpoint {
    url = "${args.promUrl}"
    basic_auth {
      username = "${args.promUser}"
      password = "${args.password}"
    }
  }
  external_labels = {
    fn0_role = "worker",
  }
}

loki.source.journal "default" {
  forward_to    = [loki.write.default.receiver]
  relabel_rules = loki.relabel.journal.rules
  labels        = { fn0_role = "worker" }
  path          = "/host/var/log/journal"
}

loki.relabel "journal" {
  forward_to = []
  rule {
    source_labels = ["__journal__systemd_unit"]
    target_label  = "unit"
  }
  rule {
    source_labels = ["__journal__hostname"]
    target_label  = "host"
  }
}

loki.write "default" {
  endpoint {
    url = "${args.lokiUrl}/loki/api/v1/push"
    basic_auth {
      username = "${args.lokiUser}"
      password = "${args.password}"
    }
  }
}
`;
}

function renderCloudInit(
  agentImageRef: string,
  agentEnv: { [k: string]: string },
  workerEnv: { [k: string]: string },
  alloyConfig: string
): string {
  const agentEnvFile = renderEnvFile({
    ...agentEnv,
    FN0_AGENT_WORKER_ENV_FILE: "/etc/fn0-worker-agent/worker-env",
  });
  const workerEnvFile = renderEnvFile(workerEnv);
  const agentSystemdUnit = `[Unit]
Description=fn0 worker agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/fn0-worker-agent/env
ExecStartPre=-/usr/bin/podman pull ${agentImageRef}
ExecStart=/usr/local/bin/fn0-worker-agent
Restart=on-failure
RestartSec=5
User=opc
AmbientCapabilities=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
`;
  const alloySystemdUnit = `[Unit]
Description=fn0 alloy host observability
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
Restart=on-failure
RestartSec=5
ExecStartPre=-/usr/bin/podman rm -f fn0-alloy
ExecStartPre=/usr/bin/podman pull ${ALLOY_IMAGE_REF}
ExecStart=/usr/bin/podman run --name fn0-alloy --rm \\
  --network host \\
  --pid host \\
  -v /:/host/root:ro,rslave \\
  -v /proc:/host/proc:ro \\
  -v /sys:/host/sys:ro \\
  -v /var/log/journal:/host/var/log/journal:ro \\
  -v /etc/machine-id:/etc/machine-id:ro \\
  -v /etc/fn0-alloy:/etc/fn0-alloy:ro \\
  ${ALLOY_IMAGE_REF} run --server.http.listen-addr=127.0.0.1:12345 /etc/fn0-alloy/config.alloy
ExecStop=/usr/bin/podman stop fn0-alloy

[Install]
WantedBy=multi-user.target
`;
  return `#!/bin/bash
set -euxo pipefail

if ! command -v podman >/dev/null 2>&1; then
  dnf install -y podman
fi

HOST_ID=$(curl -fsSH "Authorization: Bearer Oracle" http://169.254.169.254/opc/v2/instance/id)
if [ -z "$HOST_ID" ]; then
  echo "failed to fetch FN0_AGENT_HOST_ID from OCI metadata" >&2
  exit 1
fi

mkdir -p /etc/fn0-worker-agent
cat > /etc/fn0-worker-agent/env <<'EOF_AGENT_ENV'
${agentEnvFile}EOF_AGENT_ENV
echo "FN0_AGENT_HOST_ID=$HOST_ID" >> /etc/fn0-worker-agent/env
chmod 600 /etc/fn0-worker-agent/env

cat > /etc/fn0-worker-agent/worker-env <<'EOF_WORKER_ENV'
${workerEnvFile}EOF_WORKER_ENV
chown opc:opc /etc/fn0-worker-agent/worker-env
chmod 600 /etc/fn0-worker-agent/worker-env

podman pull ${agentImageRef}
agent_cid=$(podman create ${agentImageRef})
podman cp "$agent_cid:/usr/local/bin/fn0-worker-agent" /usr/local/bin/fn0-worker-agent.new
podman rm "$agent_cid"
chmod +x /usr/local/bin/fn0-worker-agent.new
mv /usr/local/bin/fn0-worker-agent.new /usr/local/bin/fn0-worker-agent

cat > /etc/systemd/system/fn0-worker-agent.service <<'EOF_AGENT_UNIT'
${agentSystemdUnit}EOF_AGENT_UNIT

mkdir -p /etc/fn0-alloy
cat > /etc/fn0-alloy/config.alloy <<'EOF_ALLOY_CFG'
${alloyConfig}EOF_ALLOY_CFG
chmod 600 /etc/fn0-alloy/config.alloy

cat > /etc/systemd/system/fn0-alloy.service <<'EOF_ALLOY_UNIT'
${alloySystemdUnit}EOF_ALLOY_UNIT

# Without lingering, opc's user-1000.slice tears down whenever the last SSH
# session ends, taking podman's conmon (and the worker container with it)
# with it. fn0-worker-agent.service stays up but its containers get SIGTERM'd.
loginctl enable-linger opc

# Oracle Linux ships firewalld enabled by default, which blocks 443 even
# though the OCI SecurityList allows it. Open the port through firewalld
# (kept in sync with the SecurityList).
firewall-cmd --permanent --add-port=443/tcp
firewall-cmd --reload

systemctl daemon-reload
systemctl enable --now fn0-worker-agent.service
systemctl enable --now fn0-alloy.service
`;
}

function renderEnvFile(env: { [k: string]: string }): string {
  const lines: string[] = [];
  for (const [k, v] of Object.entries(env)) {
    if (v === undefined || v === null) continue;
    lines.push(`${k}=${escapeEnvValue(v)}`);
  }
  return lines.length === 0 ? "" : lines.join("\n") + "\n";
}

function escapeEnvValue(v: string): string {
  return v.replace(/\n/g, "\\n");
}

const cloudflareIpv4Ranges = [
  "173.245.48.0/20",
  "103.21.244.0/22",
  "103.22.200.0/22",
  "103.31.4.0/22",
  "141.101.64.0/18",
  "108.162.192.0/18",
  "190.93.240.0/20",
  "188.114.96.0/20",
  "197.234.240.0/22",
  "198.41.128.0/17",
  "162.158.0.0/15",
  "104.16.0.0/13",
  "104.24.0.0/14",
  "172.64.0.0/13",
  "131.0.72.0/22",
];

const cloudflareIpv6Ranges = [
  "2400:cb00::/32",
  "2606:4700::/32",
  "2803:f800::/32",
  "2405:b500::/32",
  "2405:8100::/32",
  "2a06:98c0::/29",
  "2c0f:f248::/32",
];
