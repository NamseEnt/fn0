import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as tls from "@pulumi/tls";
import * as random from "@pulumi/random";
import { gzipSync } from "node:zlib";
import { CustomWorkerImage } from "./CustomWorkerImage";
import {
  CLOUDFLARE_IPV4_RANGES,
  CLOUDFLARE_IPV6_RANGES,
} from "./cloudflareIpRanges";

export interface OciFn0WorkerSiteArgs {
  region: pulumi.Input<string>;
  count: number;
  shape: pulumi.Input<string>;
  ocpus: pulumi.Input<number>;
  memoryInGbs: pulumi.Input<number>;
  workerAgentForteDb: WorkerAgentForteDbArgs;
  worker: WorkerArgs;
  websocketBearer: pulumi.Input<string>;
}

export interface WorkerAgentForteDbArgs {
  groupToken: pulumi.Input<string>;
  hostSuffix: pulumi.Input<string>;
}

export interface WorkerArgs {
  tlsOrigin: WorkerTlsOriginArgs;
  envEncryptionKeyBase64: pulumi.Input<string>;
  forteDb: WorkerForteDbArgs;
  crossProjectEnqueueAllowedCallerProjectId: pulumi.Input<string>;
  crossProjectInvokeAllowedCallerProjectId: pulumi.Input<string>;
  vault: WorkerVaultArgs;
  otlp: WorkerOtlpArgs;
  hostObservability: WorkerHostObservabilityArgs;
  bundleStorage: WorkerBundleStorageArgs;
  controlProjectId: pulumi.Input<string>;
  apex: WorkerApexArgs;
}

export interface WorkerApexArgs {
  domain: pulumi.Input<string>;
  projectId: pulumi.Input<string>;
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
  otlpUrl: pulumi.Input<string>;
  otlpUserId: pulumi.Input<string>;
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

const MANAGED_BY_TAG_VALUE = "fn0-control";

export class OciFn0WorkerSite extends pulumi.ComponentResource {
  public readonly compartmentId: pulumi.Output<string>;
  public readonly subnetId: pulumi.Output<string>;
  public readonly instanceConfigurationId: pulumi.Output<string>;
  public readonly workerImageRegistries: pulumi.Output<WorkerImageRegistry[]>;
  public readonly osImageId: pulumi.Output<string>;
  public readonly cwasmBucket: OciCwasmBucketInfo;
  public readonly queue: OciQueueInfo;
  public readonly sshPublicKey: pulumi.Output<string>;
  public readonly sshPrivateKey: pulumi.Output<string>;
  public readonly instancePoolId: pulumi.Output<string>;
  public readonly networkLoadBalancerPublicIp: pulumi.Output<string>;
  public readonly bastionId: pulumi.Output<string>;

  constructor(
    name: string,
    args: OciFn0WorkerSiteArgs,
    opts?: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:oci-fn0-worker-site", name, args, opts);

    const compartmentSuffix = new random.RandomString(
      "compartment-suffix",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this },
    ).result;

    const compartment = this.setupCompartment(compartmentSuffix);
    this.compartmentId = compartment.id;

    this.setupIdentity(compartmentSuffix, compartment);

    const { vcn, internetGateway, workerSubnet, nlbSubnet, workerSshKey } =
      this.setupNetwork(compartment);
    this.sshPublicKey = workerSshKey.publicKeyOpenssh;
    this.sshPrivateKey = workerSshKey.privateKeyPem;
    this.subnetId = workerSubnet.id;

    const bastion = this.setupBastion(
      compartmentSuffix,
      compartment,
      workerSubnet,
    );
    this.bastionId = bastion.id;

    const { availabilityDomain, imageId, customWorkerImage } =
      this.setupImage(args, compartment, vcn, internetGateway, workerSubnet);
    this.osImageId = imageId;

    const { workerRepo, workerImageRegistries } = this.setupContainerRegistry(
      args,
      compartmentSuffix,
      compartment,
    );
    this.workerImageRegistries = workerImageRegistries;

    this.cwasmBucket = this.setupCwasmBucket(
      args,
      compartmentSuffix,
      compartment,
      workerRepo,
    );

    this.queue = this.setupQueue(args, compartmentSuffix, compartment);

    const websocketPrivateKey = new tls.PrivateKey(
      "websocket-quic-key",
      { algorithm: "ECDSA", ecdsaCurve: "P256" },
      { parent: this },
    );
    const websocketCertificate = new tls.SelfSignedCert(
      "websocket-quic-certificate",
      {
        privateKeyPem: websocketPrivateKey.privateKeyPem,
        subject: { commonName: "fn0-worker.internal" },
        validityPeriodHours: 24 * 365 * 10,
        allowedUses: ["key_encipherment", "digital_signature", "server_auth"],
        dnsNames: ["fn0-worker.internal"],
      },
      { parent: this },
    );
    const metadata = this.buildInstanceMetadata(args, {
      certificatePem: websocketCertificate.certPem,
      privateKeyPem: websocketPrivateKey.privateKeyPem,
      bearer: args.websocketBearer,
    });

    const instanceConfiguration = this.setupInstanceConfiguration(
      args,
      compartment,
      workerSubnet,
      availabilityDomain,
      imageId,
      metadata,
    );
    this.instanceConfigurationId = instanceConfiguration.id;

    const { networkLoadBalancer, backendSet } =
      this.setupNetworkLoadBalancer(name, compartment, nlbSubnet);
    this.networkLoadBalancerPublicIp = networkLoadBalancer.ipAddresses.apply(
      (addrs) => addrs.find((a) => a.isPublic)?.ipAddress ?? "",
    );

    const instancePool = this.setupInstancePool(
      name,
      args,
      compartment,
      workerSubnet,
      availabilityDomain,
      instanceConfiguration,
      networkLoadBalancer,
      backendSet,
    );
    this.instancePoolId = instancePool.id;

    // customWorkerImage referenced to avoid unused-variable lint; imageId is
    // already its primary output.
    void customWorkerImage;

    this.registerOutputs({
      compartmentId: this.compartmentId,
      subnetId: this.subnetId,
      instanceConfigurationId: this.instanceConfigurationId,
      workerImageRegistries: this.workerImageRegistries,
      osImageId: this.osImageId,
      cwasmBucket: this.cwasmBucket,
      queue: this.queue,
      sshPublicKey: this.sshPublicKey,
      sshPrivateKey: this.sshPrivateKey,
      instancePoolId: this.instancePoolId,
      networkLoadBalancerPublicIp: this.networkLoadBalancerPublicIp,
      bastionId: this.bastionId,
    });
  }

  private setupCompartment(
    compartmentSuffix: pulumi.Output<string>,
  ): oci.identity.Compartment {
    return new oci.identity.Compartment(
      "compartment",
      {
        description: "Compartment for fn0 OCI Worker",
        name: pulumi.interpolate`fn0-host-${compartmentSuffix}`,
        enableDelete: true,
      },
      { parent: this },
    );
  }

  private setupIdentity(
    compartmentSuffix: pulumi.Output<string>,
    compartment: oci.identity.Compartment,
  ): void {
    const imageBuilderDynGroup = new oci.identity.DynamicGroup(
      "image-builder-dyn-group",
      {
        compartmentId: compartment.compartmentId,
        description: "Instances that can self-terminate for image building",
        matchingRule: pulumi.interpolate`ANY {instance.compartment.id = '${compartment.id}'}`,
        name: pulumi.interpolate`fn0-image-builder-${compartmentSuffix}`,
      },
      { parent: this },
    );

    new oci.identity.Policy(
      "image-builder-self-terminate-policy",
      {
        compartmentId: compartment.compartmentId,
        description: "Allow image builder instances to terminate themselves",
        statements: [
          pulumi.interpolate`Allow dynamic-group ${imageBuilderDynGroup.name} to manage instance-family in compartment id ${compartment.id}`,
        ],
      },
      { parent: this, dependsOn: [imageBuilderDynGroup] },
    );
  }

  private setupNetwork(compartment: oci.identity.Compartment): {
    vcn: oci.core.Vcn;
    internetGateway: oci.core.InternetGateway;
    workerSubnet: oci.core.Subnet;
    nlbSubnet: oci.core.Subnet;
    workerSshKey: tls.PrivateKey;
  } {
    const vcn = new oci.core.Vcn(
      "vcn",
      {
        compartmentId: compartment.id,
        isIpv6enabled: true,
        isOracleGuaAllocationEnabled: true,
        cidrBlocks: ["10.0.0.0/16"],
      },
      { parent: this },
    );

    const workerSshKey = new tls.PrivateKey(
      "worker-ssh-key",
      {
        algorithm: "RSA",
        rsaBits: 4096,
      },
      { parent: this },
    );

    const nlbSubnetCidr = "10.0.1.0/24";
    const workerSubnetCidr = "10.0.2.0/24";

    // Worker (backend) subnet: 443 only from the NLB subnet so the NLB is
    // the only external ingress. The worker-subnet-CIDR 22 rule is for the
    // Bastion: its private endpoint lives in the worker subnet, so bastion
    // sessions reach sshd from that CIDR.
    const workerSecurityList = new oci.core.SecurityList(
      "worker-security-list",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        ingressSecurityRules: [
          {
            protocol: "6",
            source: workerSubnetCidr,
            tcpOptions: { min: 22, max: 22 },
          },
          {
            protocol: "6",
            source: nlbSubnetCidr,
            tcpOptions: { min: 443, max: 443 },
          },
          {
            protocol: "17",
            source: workerSubnetCidr,
            udpOptions: { min: 18445, max: 65535 },
          },
          {
            protocol: "6",
            source: workerSubnetCidr,
            tcpOptions: { min: 18445, max: 65535 },
          },
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
      { parent: this },
    );

    // NLB subnet: 443 from Cloudflare's published edge ranges only.
    //
    // Users point their own proxied DNS records straight at this NLB, so its
    // address is published by design and cannot rely on being unknown. Every
    // legitimate request arrives through a Cloudflare edge — the platform's
    // zone or a user's — so anything else reaching 443 is either a scan or an
    // attempt to bypass the edge, and both should be dropped before they cost
    // a TLS handshake.
    const nlbSecurityList = new oci.core.SecurityList(
      "nlb-security-list",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        ingressSecurityRules: [
          ...CLOUDFLARE_IPV4_RANGES.map((source) => ({
            protocol: "6",
            source,
            tcpOptions: { min: 443, max: 443 },
          })),
          ...CLOUDFLARE_IPV6_RANGES.map((source) => ({
            protocol: "6",
            source,
            tcpOptions: { min: 443, max: 443 },
          })),
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
      { parent: this },
    );

    const internetGateway = new oci.core.InternetGateway(
      "igw",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
      },
      { parent: this },
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
      { parent: this },
    );

    const natGateway = new oci.core.NatGateway(
      "nat-gateway",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
      },
      { parent: this },
    );

    // No ::/0 rule: NAT is IPv4-only and worker VNICs don't get IPv6
    // addresses (isAssignIpv6ip: false).
    const workerNatRouteTable = new oci.core.RouteTable(
      "worker-nat-route-table",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        routeRules: [
          {
            destination: "0.0.0.0/0",
            destinationType: "CIDR_BLOCK",
            networkEntityId: natGateway.id,
          },
        ],
      },
      { parent: this },
    );

    const workerSubnet = new oci.core.Subnet(
      "worker-subnet",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        ipv4cidrBlocks: [workerSubnetCidr],
        prohibitInternetIngress: false,
        prohibitPublicIpOnVnic: false,
        securityListIds: [workerSecurityList.id],
        routeTableId: workerNatRouteTable.id,
      },
      { parent: this },
    );

    const nlbSubnet = new oci.core.Subnet(
      "nlb-subnet",
      {
        compartmentId: compartment.id,
        vcnId: vcn.id,
        ipv4cidrBlocks: [nlbSubnetCidr],
        prohibitInternetIngress: false,
        prohibitPublicIpOnVnic: false,
        securityListIds: [nlbSecurityList.id],
        routeTableId: routeTable.id,
      },
      { parent: this },
    );

    return { vcn, internetGateway, workerSubnet, nlbSubnet, workerSshKey };
  }

  // Debug SSH path replacing the workers' public-IP + world-open port 22
  // (issue #42): sessions are created on demand via IAM (max TTL 3h) and
  // tunnel to the workers' private IPs, so nothing stays publicly reachable.
  private setupBastion(
    compartmentSuffix: pulumi.Output<string>,
    compartment: oci.identity.Compartment,
    workerSubnet: oci.core.Subnet,
  ): oci.bastion.Bastion {
    return new oci.bastion.Bastion(
      "worker-bastion",
      {
        bastionType: "STANDARD",
        // OCI bastion names reject hyphens, so pulumi auto-naming can't be
        // used here.
        name: pulumi.interpolate`fn0worker${compartmentSuffix}`,
        compartmentId: compartment.id,
        targetSubnetId: workerSubnet.id,
        clientCidrBlockAllowLists: ["0.0.0.0/0"],
        maxSessionTtlInSeconds: 10800,
      },
      { parent: this },
    );
  }

  private setupImage(
    args: OciFn0WorkerSiteArgs,
    compartment: oci.identity.Compartment,
    vcn: oci.core.Vcn,
    internetGateway: oci.core.InternetGateway,
    workerSubnet: oci.core.Subnet,
  ): {
    availabilityDomain: pulumi.Output<string>;
    imageId: pulumi.Output<string>;
    customWorkerImage: CustomWorkerImage;
  } {
    void args;
    void workerSubnet;
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
        }),
    );

    const baseImageId =
      "ocid1.image.oc1.ap-osaka-1.aaaaaaaa3gpxiv2x5tradmcp4mbzhkh75otz4fqi5qsebxyhcjg2jzlmicna";

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
      { parent: this },
    );

    const imageId = customWorkerImage.imageId;

    return { availabilityDomain, imageId, customWorkerImage };
  }

  private setupInstanceConfiguration(
    args: OciFn0WorkerSiteArgs,
    compartment: oci.identity.Compartment,
    workerSubnet: oci.core.Subnet,
    availabilityDomain: pulumi.Output<string>,
    imageId: pulumi.Output<string>,
    metadata: pulumi.Output<{ [k: string]: string }>,
  ): oci.core.InstanceConfiguration {
    return new oci.core.InstanceConfiguration(
      "instance-configuration",
      {
        compartmentId: compartment.id,
        instanceDetails: {
          instanceType: "compute",
          launchDetails: {
            compartmentId: compartment.id,
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
              subnetId: workerSubnet.id,
              assignPublicIp: false,
            },
            metadata,
            freeformTags: {
              managed_by: MANAGED_BY_TAG_VALUE,
              fn0_role: "worker",
            },
          },
          // Sized so that two instances stay inside the 200 GB Always Free
          // block storage allowance: a roll runs the pool at size 2, and each
          // instance carries its own ~47 GB boot volume plus this volume.
          blockVolumes: [
            {
              createDetails: {
                availabilityDomain,
                compartmentId: compartment.id,
                sizeInGbs: `${ALLOY_QUEUE_VOLUME_SIZE_IN_GBS}`,
                vpusPerGb: "10",
                displayName: "fn0-worker-alloy-queue",
                freeformTags: {
                  managed_by: MANAGED_BY_TAG_VALUE,
                  fn0_role: "worker",
                },
              },
              attachDetails: {
                type: "paravirtualized",
                device: ALLOY_QUEUE_DEVICE,
                displayName: "fn0-worker-alloy-queue",
              },
            },
          ],
        },
      },
      { parent: this },
    );
  }

  private setupContainerRegistry(
    args: OciFn0WorkerSiteArgs,
    compartmentSuffix: pulumi.Output<string>,
    compartment: oci.identity.Compartment,
  ): {
    workerRepo: oci.artifacts.ContainerRepository;
    workerImageRegistries: pulumi.Output<WorkerImageRegistry[]>;
  } {
    const workerRepo = new oci.artifacts.ContainerRepository(
      "worker-repo",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-worker-${compartmentSuffix}`,
        isPublic: true,
      },
      { parent: this, retainOnDelete: false },
    );

    new oci.artifacts.ContainerRepository(
      "worker-agent-repo",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-worker-${compartmentSuffix}-agent`,
        isPublic: true,
      },
      { parent: this, retainOnDelete: false },
    );

    new oci.artifacts.ContainerRepository(
      "worker-proxy-repo",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-worker-${compartmentSuffix}-proxy`,
        isPublic: true,
      },
      { parent: this, retainOnDelete: false },
    );

    const workerDockerUser = new oci.identity.User(
      "worker-docker-user",
      {
        name: pulumi.interpolate`fn0-worker-docker-${compartmentSuffix}`,
        description: "User for fn0-worker image push",
      },
      { parent: this },
    );

    const workerDockerGroup = new oci.identity.Group(
      "worker-docker-group",
      {
        name: pulumi.interpolate`fn0-worker-pushers-${compartmentSuffix}`,
        description: "Group allowed to push worker images",
      },
      { parent: this },
    );

    new oci.identity.UserGroupMembership(
      "worker-docker-membership",
      {
        userId: workerDockerUser.id,
        groupId: workerDockerGroup.id,
      },
      { parent: this },
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
      { dependsOn: [workerDockerGroup], parent: this },
    );

    const workerAuthToken = new oci.identity.AuthToken(
      "worker-auth-token",
      {
        userId: workerDockerUser.id,
        description: "AuthToken for fn0-worker image push",
      },
      { parent: this },
    );

    const registryUrl = pulumi.interpolate`ocir.${args.region}.oci.oraclecloud.com`;

    const workerImageRegistries = pulumi
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

    return { workerRepo, workerImageRegistries };
  }

  private setupCwasmBucket(
    args: OciFn0WorkerSiteArgs,
    compartmentSuffix: pulumi.Output<string>,
    compartment: oci.identity.Compartment,
    workerRepo: oci.artifacts.ContainerRepository,
  ): OciCwasmBucketInfo {
    const cwasmBucketUser = new oci.identity.User(
      "cwasm-bucket-user",
      {
        name: pulumi.interpolate`fn0-cwasm-${compartmentSuffix}`,
        description: "User for cwasm S3-compatible access",
      },
      { parent: this },
    );

    const cwasmBucketGroup = new oci.identity.Group(
      "cwasm-bucket-group",
      {
        name: pulumi.interpolate`fn0-cwasm-group-${compartmentSuffix}`,
        description: "Group for cwasm bucket access",
      },
      { parent: this },
    );

    new oci.identity.UserGroupMembership(
      "cwasm-bucket-membership",
      {
        userId: cwasmBucketUser.id,
        groupId: cwasmBucketGroup.id,
      },
      { parent: this },
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
      { dependsOn: [cwasmBucketGroup], parent: this },
    );

    const customerSecretKey = new oci.identity.CustomerSecretKey(
      "cwasm-s3-key",
      {
        userId: cwasmBucketUser.id,
        displayName: "cwasm-s3-compatible-key",
      },
      { parent: this },
    );

    const ociObjectStorageBucket = new oci.objectstorage.Bucket(
      "cwasm-bucket",
      {
        compartmentId: compartment.id,
        name: pulumi.interpolate`fn0-cwasm-${compartmentSuffix}`,
        namespace: workerRepo.namespace,
        accessType: "NoPublicAccess",
      },
      { parent: this },
    );

    return {
      endpoint: pulumi.interpolate`https://${workerRepo.namespace}.compat.objectstorage.${args.region}.oraclecloud.com`,
      region: pulumi.output(args.region),
      bucketName: ociObjectStorageBucket.name,
      accessKeyId: customerSecretKey.id,
      secretAccessKey: customerSecretKey.key,
      namespace: workerRepo.namespace,
    };
  }

  private setupQueue(
    args: OciFn0WorkerSiteArgs,
    compartmentSuffix: pulumi.Output<string>,
    compartment: oci.identity.Compartment,
  ): OciQueueInfo {
    const queue = new oci.queue.Queue(
      "queue",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-queue-${compartmentSuffix}`,
      },
      { parent: this },
    );

    const queueUser = new oci.identity.User(
      "queue-user",
      {
        name: pulumi.interpolate`fn0-queue-${compartmentSuffix}`,
        description: "User for fn0 queue produce/consume",
      },
      { parent: this },
    );

    const queueGroup = new oci.identity.Group(
      "queue-group",
      {
        name: pulumi.interpolate`fn0-queue-group-${compartmentSuffix}`,
        description: "Group for fn0 queue access",
      },
      { parent: this },
    );

    new oci.identity.UserGroupMembership(
      "queue-membership",
      {
        userId: queueUser.id,
        groupId: queueGroup.id,
      },
      { parent: this },
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
      { dependsOn: [queueGroup], parent: this },
    );

    const queueApiPrivateKey = new tls.PrivateKey(
      "queue-api-key-pair",
      {
        algorithm: "RSA",
        rsaBits: 2048,
      },
      { parent: this },
    );

    const queueApiKey = new oci.identity.ApiKey(
      "queue-api-key",
      {
        userId: queueUser.id,
        keyValue: queueApiPrivateKey.publicKeyPem,
      },
      { parent: this },
    );

    return {
      ocid: queue.id,
      messagesEndpoint: queue.messagesEndpoint,
      region: pulumi.output(args.region),
      ociUserId: queueUser.id,
      ociTenancyId: queueUser.compartmentId,
      ociFingerprint: queueApiKey.fingerprint,
      ociPrivateKeyBase64: queueApiPrivateKey.privateKeyPemPkcs8.apply((pem) =>
        Buffer.from(pem).toString("base64"),
      ),
    };
  }

  private buildInstanceMetadata(
    args: OciFn0WorkerSiteArgs,
    websocketTransport: {
      certificatePem: pulumi.Input<string>;
      privateKeyPem: pulumi.Input<string>;
      bearer: pulumi.Input<string>;
    },
  ): pulumi.Output<{ [k: string]: string }> {
    // Mutable tag: agent_image_pull_poller relies on `:latest` moving to a new
    // digest to self-update.
    const agentImageRef = this.workerImageRegistries.apply((regs) => {
      if (regs.length === 0) {
        throw new Error("workerImageRegistries is empty");
      }
      const r = regs[0];
      return `${r.url}/${r.repository}-agent:latest`;
    });

    const proxyImageRef = this.workerImageRegistries.apply((regs) => {
      if (regs.length === 0) {
        throw new Error("workerImageRegistries is empty");
      }
      const r = regs[0];
      return `${r.url}/${r.repository}-proxy:latest`;
    });

    const agentEnv = buildWorkerAgentEnv(args.workerAgentForteDb);
    const workerEnv = buildWorkerEnv(
      args.worker,
      this.cwasmBucket,
      this.queue,
      websocketTransport,
    );
    const alloyConfig = pulumi
      .all([
        args.worker.hostObservability.prometheusUrl,
        args.worker.hostObservability.prometheusUserId,
        args.worker.hostObservability.lokiUrl,
        args.worker.hostObservability.lokiUserId,
        args.worker.hostObservability.otlpUrl,
        args.worker.hostObservability.otlpUserId,
      ])
      .apply(([promUrl, promUser, lokiUrl, lokiUser, otlpUrl, otlpUser]) =>
        renderAlloyConfig({ promUrl, promUser, lokiUrl, lokiUser, otlpUrl, otlpUser }),
      );
    const alloyEnvFile = pulumi
      .output(args.worker.hostObservability.basicAuthPassword)
      .apply((password) =>
        renderEnvFile({ FN0_GRAFANA_PASSWORD: password }),
      );

    const cloudInit = pulumi
      .all([agentImageRef, proxyImageRef, agentEnv, workerEnv, alloyConfig, alloyEnvFile])
      .apply(([agentImageRef, proxyImageRef, agentEnv, workerEnv, alloyConfig, alloyEnvFile]) =>
        renderCloudInit(
          agentImageRef,
          proxyImageRef,
          agentEnv,
          workerEnv,
          alloyConfig,
          alloyEnvFile,
        ),
      );
    const userData = cloudInit.apply((s) =>
      gzipSync(Buffer.from(s, "utf8")).toString("base64"),
    );

    return pulumi
      .all([userData, this.sshPublicKey])
      .apply(([ud, ssh]) => {
        const m: { [k: string]: string } = { user_data: ud };
        if (ssh) m["ssh_authorized_keys"] = ssh;
        return m;
      });
  }

  private setupNetworkLoadBalancer(
    name: string,
    compartment: oci.identity.Compartment,
    nlbSubnet: oci.core.Subnet,
  ): {
    networkLoadBalancer: oci.networkloadbalancer.NetworkLoadBalancer;
    backendSet: oci.networkloadbalancer.BackendSet;
  } {
    // Ephemeral public IP: OCI allocates one for the NLB. If the NLB is ever
    // recreated the IP changes, but Pulumi forwards the new IP into the DNS A
    // records below so callers (Cloudflare proxy) follow automatically.
    const networkLoadBalancer = new oci.networkloadbalancer.NetworkLoadBalancer(
      "nlb",
      {
        compartmentId: compartment.id,
        displayName: `${name}-nlb`,
        subnetId: nlbSubnet.id,
        isPrivate: false,
      },
      { parent: this },
    );

    const backendSet = new oci.networkloadbalancer.BackendSet(
      "nlb-backend-set",
      {
        networkLoadBalancerId: networkLoadBalancer.id,
        name: "workers",
        // TWO_TUPLE = hash(sourceIp, destIp); destIp is the NLB public IP (fixed),
        // so this effectively becomes source-IP affinity. Used to keep the same
        // client (= same Cloudflare edge IP from NLB's view) pinned to the same
        // worker, preserving warm WASM/JS instance caches in CodeExecutor.
        // Hash is computed on the incoming connection, independent of
        // isPreserveSource below.
        policy: "TWO_TUPLE",
        // Full NAT: workers see the NLB private IP as source, so the backend
        // SecurityList only has to allow the NLB subnet CIDR (no Cloudflare IP
        // range maintenance). Clients' real IPs still reach workers via the
        // Cloudflare-set HTTP headers (CF-Connecting-IP / X-Forwarded-For), so
        // this loses nothing the application cares about.
        isPreserveSource: false,
        healthChecker: {
          protocol: "HTTP",
          port: 443,
          urlPath: "/health",
          returnCode: 200,
        },
      },
      { parent: this },
    );

    new oci.networkloadbalancer.Listener(
      "nlb-listener-tcp-443",
      {
        networkLoadBalancerId: networkLoadBalancer.id,
        name: "tcp-443",
        defaultBackendSetName: backendSet.name,
        port: 443,
        protocol: "TCP",
      },
      { parent: this },
    );

    return { networkLoadBalancer, backendSet };
  }

  private setupInstancePool(
    name: string,
    args: OciFn0WorkerSiteArgs,
    compartment: oci.identity.Compartment,
    workerSubnet: oci.core.Subnet,
    availabilityDomain: pulumi.Output<string>,
    instanceConfiguration: oci.core.InstanceConfiguration,
    networkLoadBalancer: oci.networkloadbalancer.NetworkLoadBalancer,
    backendSet: oci.networkloadbalancer.BackendSet,
  ): oci.core.InstancePool {
    const instancePool = new oci.core.InstancePool(
      "instance-pool",
      {
        compartmentId: compartment.id,
        instanceConfigurationId: instanceConfiguration.id,
        displayName: name,
        size: args.count,
        placementConfigurations: [
          {
            availabilityDomain,
            primaryVnicSubnets: {
              subnetId: workerSubnet.id,
              isAssignIpv6ip: false,
              ipv6addressIpv6subnetCidrPairDetails: [],
            },
          },
        ],
        loadBalancers: [
          {
            loadBalancerId: networkLoadBalancer.id,
            backendSetName: backendSet.name,
            port: 443,
            vnicSelection: "PrimaryVnic",
          },
        ],
        freeformTags: {
          managed_by: MANAGED_BY_TAG_VALUE,
          fn0_role: "worker",
        },
      },
      { parent: this },
    );

    new oci.autoscaling.AutoScalingConfiguration(
      "instance-pool-autoscaling",
      {
        compartmentId: compartment.id,
        displayName: `${name}-autoscaling`,
        coolDownInSeconds: 300,
        isEnabled: true,
        autoScalingResources: {
          id: instancePool.id,
          type: "instancePool",
        },
        policies: [
          {
            policyType: "threshold",
            displayName: "cpu-threshold",
            capacity: {
              initial: args.count,
              min: args.count,
              // OCI rejects max == min ("Maximum capacity must be greater than
              // the minimum"). Headroom of +1 keeps the policy schema valid
              // while staying within the Always Free A1 quota for now.
              max: args.count + 1,
            },
            rules: [
              {
                displayName: "scale-out-cpu-70",
                action: { type: "CHANGE_COUNT_BY", value: 1 },
                metric: {
                  metricType: "CPU_UTILIZATION",
                  threshold: { operator: "GT", value: 70 },
                },
              },
              {
                displayName: "scale-in-cpu-30",
                action: { type: "CHANGE_COUNT_BY", value: -1 },
                metric: {
                  metricType: "CPU_UTILIZATION",
                  threshold: { operator: "LT", value: 30 },
                },
              },
            ],
          },
        ],
      },
      { parent: this, dependsOn: [instancePool], deleteBeforeReplace: true },
    );

    return instancePool;
  }
}

function buildWorkerAgentEnv(
  forteDb: WorkerAgentForteDbArgs,
): pulumi.Output<{ [k: string]: string }> {
  const base: { [k: string]: pulumi.Input<string> } = {
    TURSO_GROUP_TOKEN: forteDb.groupToken,
    TURSO_DB_HOST_SUFFIX: forteDb.hostSuffix,
  };
  return resolveEnvMap(base);
}

function buildWorkerEnv(
  worker: WorkerArgs,
  cwasmBucket: OciCwasmBucketInfo,
  queue: OciQueueInfo,
  websocketTransport: {
    certificatePem: pulumi.Input<string>;
    privateKeyPem: pulumi.Input<string>;
    bearer: pulumi.Input<string>;
  },
): pulumi.Output<{ [k: string]: string }> {
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

    // No R2 credentials: every project's storage is its owner's, resolved per
    // request from the worker manifest.
    FN0_CONTROL_PROJECT_ID: worker.controlProjectId,

    FN0_APEX_DOMAIN: worker.apex.domain,
    FN0_APEX_PROJECT_ID: worker.apex.projectId,

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

    FN0_CROSS_PROJECT_ENQUEUE_OCID: queue.ocid,
    FN0_CROSS_PROJECT_ENQUEUE_MESSAGES_ENDPOINT: queue.messagesEndpoint,
    FN0_CROSS_PROJECT_ENQUEUE_OCI_USER_ID: queue.ociUserId,
    FN0_CROSS_PROJECT_ENQUEUE_OCI_TENANCY_ID: queue.ociTenancyId,
    FN0_CROSS_PROJECT_ENQUEUE_OCI_FINGERPRINT: queue.ociFingerprint,
    FN0_CROSS_PROJECT_ENQUEUE_OCI_PRIVATE_KEY_BASE64: queue.ociPrivateKeyBase64,
    FN0_CROSS_PROJECT_ENQUEUE_ALLOWED_CALLER_PROJECT_ID:
      worker.crossProjectEnqueueAllowedCallerProjectId,

    FN0_CROSS_PROJECT_INVOKE_ALLOWED_CALLER_PROJECT_ID:
      worker.crossProjectInvokeAllowedCallerProjectId,

    FN0_VAULT_CRYPTO_ENDPOINT: worker.vault.cryptoEndpoint,
    FN0_VAULT_KEY_OCID: worker.vault.keyOcid,
    FN0_VAULT_REGION: worker.vault.region,
    FN0_VAULT_ALLOWED_SUBDOMAIN: worker.vault.allowedSubdomain,

    OTLP_ENDPOINT: "http://127.0.0.1:4318",
    OTLP_BASIC_AUTH: worker.otlp.basicAuth,
    FN0_OTLP_TARGET_HOST: "127.0.0.1:4318",
    FN0_OTLP_TARGET_SCHEME: "http",
    FN0_OTLP_TARGET_PATH_PREFIX: "",
    FN0_OTLP_AUTH: "",

    FN0_WEBSOCKET_QUIC_CERT_PEM_BASE64: pulumi
      .output(websocketTransport.certificatePem)
      .apply((certificatePem) => Buffer.from(certificatePem, "utf8").toString("base64")),
    FN0_WEBSOCKET_QUIC_KEY_PEM_BASE64: pulumi
      .output(websocketTransport.privateKeyPem)
      .apply((privateKeyPem) => Buffer.from(privateKeyPem, "utf8").toString("base64")),
    FN0_WEBSOCKET_QUIC_SERVER_NAME: "fn0-worker.internal",
    FN0_WEBSOCKET_QUIC_BEARER: websocketTransport.bearer,
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
const ALLOY_QUEUE_VOLUME_SIZE_IN_GBS = 50;
const ALLOY_QUEUE_DEVICE = "/dev/oracleoci/oraclevdb";
const ALLOY_STORAGE_PATH = "/var/lib/alloy";
const ALLOY_OTLP_QUEUE_SIZE_IN_BYTES = 32 * 1024 * 1024 * 1024;
// A sample is dropped from the WAL once it is this old even if it was never
// delivered. This, not the volume size, is what bounds how long a metrics
// backend outage the host metrics can survive; upstream defaults it to 8h.
const ALLOY_PROMETHEUS_WAL_MAX_KEEPALIVE = "168h";

function renderAlloyConfig(args: {
  promUrl: string;
  promUser: string;
  lokiUrl: string;
  lokiUser: string;
  otlpUrl: string;
  otlpUser: string;
}): string {
  // Each scrape target gets an instance=<ocid> label via discovery.relabel
  // because Prometheus external_labels do not override labels a target
  // already carries (the exporters set instance=<hostname:port> by default).
  return `prometheus.exporter.unix "node" {
  set_collectors = ["cpu", "diskstats", "filesystem", "loadavg", "meminfo", "netdev", "netstat", "vmstat", "uname", "time"]
  rootfs_path     = "/host/root"
  procfs_path     = "/host/proc"
  sysfs_path      = "/host/sys"
}

discovery.relabel "node" {
  targets = prometheus.exporter.unix.node.targets
  rule {
    target_label = "instance"
    replacement  = sys.env("FN0_HOST_OCID")
  }
}

prometheus.scrape "node" {
  targets    = discovery.relabel.node.output
  forward_to = [prometheus.remote_write.default.receiver]
}

prometheus.remote_write "default" {
  endpoint {
    url = "${args.promUrl}"
    basic_auth {
      username = "${args.promUser}"
      password = sys.env("FN0_GRAFANA_PASSWORD")
    }
  }
  external_labels = {
    fn0_role = "worker",
  }
  wal {
    max_keepalive_time = "${ALLOY_PROMETHEUS_WAL_MAX_KEEPALIVE}"
  }
}

loki.source.journal "default" {
  forward_to    = [loki.write.default.receiver]
  relabel_rules = loki.relabel.journal.rules
  labels        = {
    fn0_role = "worker",
    instance = sys.env("FN0_HOST_OCID"),
  }
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
  rule {
    source_labels = ["__journal_container_name"]
    target_label  = "container"
  }
}

loki.write "default" {
  endpoint {
    url = "${args.lokiUrl}/loki/api/v1/push"
    basic_auth {
      username = "${args.lokiUser}"
      password = sys.env("FN0_GRAFANA_PASSWORD")
    }
  }
}

otelcol.receiver.otlp "default" {
  http {
    endpoint         = "127.0.0.1:4318"
    include_metadata = true
  }
  grpc {
    endpoint         = "127.0.0.1:4317"
    include_metadata = true
  }
  output {
    metrics = [otelcol.processor.attributes.fn0_stamping.input]
    logs    = [otelcol.processor.attributes.fn0_stamping.input]
    traces  = [otelcol.processor.attributes.fn0_stamping.input]
  }
}

otelcol.processor.attributes "fn0_stamping" {
  action {
    key          = "fn0.project_id"
    action       = "upsert"
    from_context = "X-Fn0-Project-Id"
  }
  output {
    metrics = [otelcol.processor.deltatocumulative.default.input]
    logs    = [otelcol.processor.batch.default.input]
    traces  = [otelcol.processor.batch.default.input]
  }
}

otelcol.processor.deltatocumulative "default" {
  max_stale = "5m"
  output {
    metrics = [otelcol.processor.batch.default.input]
  }
}

otelcol.processor.batch "default" {
  output {
    metrics = [otelcol.exporter.otlphttp.default.input]
    logs    = [otelcol.exporter.otlphttp.default.input]
    traces  = [otelcol.exporter.otlphttp.default.input]
  }
}

otelcol.storage.file "queue" {
  directory = "${ALLOY_STORAGE_PATH}/otlp-queue"
}

otelcol.exporter.otlphttp "default" {
  client {
    endpoint = "${args.otlpUrl}"
    auth     = otelcol.auth.basic.grafana.handler
  }
  sending_queue {
    storage    = otelcol.storage.file.queue.handler
    sizer      = "bytes"
    queue_size = ${ALLOY_OTLP_QUEUE_SIZE_IN_BYTES}
  }
}

otelcol.auth.basic "grafana" {
  username = "${args.otlpUser}"
  password = sys.env("FN0_GRAFANA_PASSWORD")
}
`;
}

function renderCloudInit(
  agentImageRef: string,
  proxyImageRef: string,
  agentEnv: { [k: string]: string },
  workerEnv: { [k: string]: string },
  alloyConfig: string,
  alloyEnvFile: string,
): string {
  const agentEnvFile = renderEnvFile({
    ...agentEnv,
    FN0_WORKER_ENV_FILE: "/etc/fn0-worker-agent/worker-env",
  });
  const workerEnvFile = renderEnvFile(workerEnv);
  // Hardcoded UID 1000: opc is always uid 1000 on Oracle Linux cloud images.
  // --security-opt label=disable: OL ships SELinux enforcing, which otherwise
  // blocks the container from the bind-mounted podman socket and config dirs.
  const agentSystemdUnit = `[Unit]
Description=fn0 worker agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=opc
ExecStartPre=-/usr/bin/podman rm -f fn0-agent
ExecStartPre=/usr/bin/podman pull ${agentImageRef}
ExecStart=/usr/bin/podman run --name fn0-agent --rm \\
  --security-opt label=disable \\
  --network host \\
  -e FN0_WORKER_AGENT_PODMAN_REMOTE_URL=unix:///run/podman/podman.sock \\
  -e FN0_WORKER_AGENT_SOFT_SHUTDOWN_ON_SIGTERM=1 \\
  -e FN0_WORKER_AGENT_IMAGE_REF=${agentImageRef} \\
  -e FN0_WORKER_AGENT_PROXY_TARGET_FILE=/etc/fn0-worker-proxy/target \\
  --env-file /etc/fn0-worker-agent/env \\
  -v /run/user/1000/podman/podman.sock:/run/podman/podman.sock \\
  -v /etc/fn0-worker-agent:/etc/fn0-worker-agent:ro \\
  -v /etc/fn0-worker-proxy:/etc/fn0-worker-proxy \\
  ${agentImageRef}
ExecStop=/usr/bin/podman stop -t 30 fn0-agent
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
`;
  const proxySystemdUnit = `[Unit]
Description=fn0 worker proxy
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=opc
ExecStartPre=-/usr/bin/podman rm -f fn0-proxy
ExecStartPre=/usr/bin/podman pull ${proxyImageRef}
ExecStart=/usr/bin/podman run --name fn0-proxy --rm \\
  --security-opt label=disable \\
  --network host \\
  -e FN0_WORKER_PROXY_TARGET_FILE=/etc/fn0-worker-proxy/target \\
  -v /etc/fn0-worker-proxy:/etc/fn0-worker-proxy:ro \\
  ${proxyImageRef}
ExecStop=/usr/bin/podman stop -t 10 fn0-proxy
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
`;
  // --security-opt label=disable: OL SELinux enforcing otherwise blocks the
  // alloy container from reading host /var/log/journal, /sys, etc.
  const alloySystemdUnit = `[Unit]
Description=fn0 alloy host observability
After=network-online.target
Wants=network-online.target
RequiresMountsFor=${ALLOY_STORAGE_PATH}

[Service]
Type=simple
Restart=on-failure
RestartSec=5
EnvironmentFile=/etc/fn0-alloy/env
ExecStartPre=-/usr/bin/podman rm -f fn0-alloy
ExecStartPre=/usr/bin/podman pull ${ALLOY_IMAGE_REF}
ExecStart=/usr/bin/podman run --name fn0-alloy --rm \\
  --security-opt label=disable \\
  --network host \\
  --env FN0_HOST_OCID \\
  --env FN0_GRAFANA_PASSWORD \\
  -v /:/host/root:ro,rslave \\
  -v /proc:/host/proc:ro \\
  -v /sys:/host/sys:ro \\
  -v /var/log/journal:/host/var/log/journal:ro \\
  -v /etc/machine-id:/etc/machine-id:ro \\
  -v /etc/fn0-alloy:/etc/fn0-alloy:ro \\
  -v ${ALLOY_STORAGE_PATH}:${ALLOY_STORAGE_PATH} \\
  ${ALLOY_IMAGE_REF} run --stability.level=experimental --server.http.listen-addr=127.0.0.1:12345 --storage.path=${ALLOY_STORAGE_PATH} /etc/fn0-alloy/config.alloy
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
  echo "failed to fetch FN0_WORKER_AGENT_HOST_ID from OCI metadata" >&2
  exit 1
fi

mkdir -p /etc/fn0-worker-agent
cat > /etc/fn0-worker-agent/env <<'EOF_AGENT_ENV'
${agentEnvFile}EOF_AGENT_ENV
echo "FN0_WORKER_AGENT_HOST_ID=$HOST_ID" >> /etc/fn0-worker-agent/env
chown opc:opc /etc/fn0-worker-agent/env
chmod 600 /etc/fn0-worker-agent/env

cat > /etc/fn0-worker-agent/worker-env <<'EOF_WORKER_ENV'
${workerEnvFile}EOF_WORKER_ENV
chown opc:opc /etc/fn0-worker-agent/worker-env
chmod 600 /etc/fn0-worker-agent/worker-env

mkdir -p /etc/fn0-worker-proxy
chown opc:opc /etc/fn0-worker-proxy
chmod 755 /etc/fn0-worker-proxy

loginctl enable-linger opc

opc_uid="$(id -u opc)"
for _ in $(seq 1 30); do
  if [ -d "/run/user/$opc_uid" ]; then break; fi
  sleep 1
done

sudo -u opc XDG_RUNTIME_DIR="/run/user/$opc_uid" \\
  systemctl --user enable --now podman.socket

cat > /etc/systemd/system/fn0-worker-agent.service <<'EOF_AGENT_UNIT'
${agentSystemdUnit}EOF_AGENT_UNIT

cat > /etc/systemd/system/fn0-worker-proxy.service <<'EOF_PROXY_UNIT'
${proxySystemdUnit}EOF_PROXY_UNIT

{
  for _ in $(seq 1 60); do
    if [ -b "${ALLOY_QUEUE_DEVICE}" ]; then break; fi
    sleep 2
  done
  if ! blkid "${ALLOY_QUEUE_DEVICE}" >/dev/null 2>&1; then
    mkfs.xfs "${ALLOY_QUEUE_DEVICE}"
  fi
  mkdir -p ${ALLOY_STORAGE_PATH}
  alloy_queue_uuid="$(blkid -s UUID -o value "${ALLOY_QUEUE_DEVICE}")"
  if ! grep -q "$alloy_queue_uuid" /etc/fstab; then
    echo "UUID=$alloy_queue_uuid ${ALLOY_STORAGE_PATH} xfs defaults,nofail 0 2" >> /etc/fstab
  fi
  systemctl daemon-reload
  mountpoint -q ${ALLOY_STORAGE_PATH} || mount ${ALLOY_STORAGE_PATH}
} || echo "alloy queue volume setup failed; alloy will not start" >&2

mkdir -p /etc/fn0-alloy
cat > /etc/fn0-alloy/config.alloy <<'EOF_ALLOY_CFG'
${alloyConfig}EOF_ALLOY_CFG
chmod 600 /etc/fn0-alloy/config.alloy

cat > /etc/fn0-alloy/env <<'EOF_ALLOY_ENV'
${alloyEnvFile}EOF_ALLOY_ENV
echo "FN0_HOST_OCID=$HOST_ID" >> /etc/fn0-alloy/env
chmod 600 /etc/fn0-alloy/env

cat > /etc/systemd/system/fn0-alloy.service <<'EOF_ALLOY_UNIT'
${alloySystemdUnit}EOF_ALLOY_UNIT

mkdir -p /etc/systemd/journald.conf.d
cat > /etc/systemd/journald.conf.d/fn0.conf <<'EOF_JOURNALD'
[Journal]
Storage=persistent
SystemMaxUse=512M
EOF_JOURNALD
mkdir -p /var/log/journal
systemd-tmpfiles --create --prefix /var/log/journal
systemctl restart systemd-journald
journalctl --flush

firewall-cmd --permanent --add-port=443/tcp
firewall-cmd --permanent --add-port=18445-65535/udp
firewall-cmd --reload

echo 'net.ipv4.ip_unprivileged_port_start=443' > /etc/sysctl.d/90-fn0-worker-proxy.conf
sysctl -p /etc/sysctl.d/90-fn0-worker-proxy.conf

systemctl daemon-reload
systemctl enable --now fn0-worker-proxy.service
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
