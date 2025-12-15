import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";

export interface OciHeadQuarterArgs {
  region: pulumi.Input<string>;
}

export class OciHeadQuarter extends pulumi.ComponentResource {
  ipv6cidrBlocks: pulumi.Output<string[]>;

  constructor(
    name: string,
    args: OciHeadQuarterArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:oci-head-quarter", name, args, opts);

    const compartment = new oci.identity.Compartment(
      "compartment",
      {
        description: "Compartment for fn0 OCI Head Quarter",
        name: `fn0-head-quater-${name}`,
      },
      { parent: this }
    );

    const { nlb, vcn, subnet } = setVcn(this, compartment);

    this.ipv6cidrBlocks = vcn.ipv6cidrBlocks;

    const imageId = compartment.id.apply((compartmentId) =>
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

    const instanceConfiguration = new oci.core.InstanceConfiguration(
      "instance-configuration",
      {
        compartmentId: compartment.id,
        instanceDetails: {
          instanceType: "compute",
          launchDetails: {
            shape: "VM.Standard.A1.Flex",
            shapeConfig: {
              ocpus: 1,
              memoryInGbs: 6,
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

    const backendSet = new oci.networkloadbalancer.BackendSet(
      "nlb-backend-set",
      {
        networkLoadBalancerId: nlb.id,
        healthChecker: {
          protocol: "HTTP",
          intervalInMillis: 10000,
          port: 8080,
          retries: 3,
          returnCode: 200,
          timeoutInMillis: 3000,
        },
        policy: "FIVE_TUPLE",
      }
    );

    new oci.core.InstancePool("instance-pool", {
      compartmentId: compartment.id,
      instanceConfigurationId: instanceConfiguration.id,
      placementConfigurations: [
        {
          availabilityDomain: subnet.availabilityDomain,
          primarySubnetId: subnet.id,
        },
      ],
      loadBalancers: [
        {
          backendSetName: backendSet.name,
          loadBalancerId: backendSet.networkLoadBalancerId,
          port: 8080,
          vnicSelection: "PrimaryVnic",
        },
      ],
      size: 1,
    });
  }
}

function setVcn(self: pulumi.Resource, compartment: oci.identity.Compartment) {
  const vcn = new oci.core.Vcn(
    "vcn",
    {
      compartmentId: compartment.id,
      isIpv6enabled: true,
      isOracleGuaAllocationEnabled: true,
      cidrBlocks: ["10.0.0.0/16"],
    },
    { parent: self }
  );

  const internetGateway = new oci.core.InternetGateway(
    "igw",
    {
      compartmentId: compartment.id,
      vcnId: vcn.id,
    },
    { parent: self }
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
    { parent: self }
  );

  const subnet = new oci.core.Subnet(
    "subnet",
    {
      compartmentId: compartment.id,
      availabilityDomain: compartment.id.apply((compartmentId) =>
        oci.identity
          .getAvailabilityDomain({
            adNumber: 1,
            compartmentId,
          })
          .then((x) => x.name)
      ),
      vcnId: vcn.id,
      ipv4cidrBlocks: ["10.0.0.0/24"],
      ipv6cidrBlocks: vcn.ipv6cidrBlocks.apply((x) =>
        x.map((x) => x.replace("/56", "/64"))
      ),
      prohibitInternetIngress: true,
      prohibitPublicIpOnVnic: false,
      routeTableId: routeTable.id,
    },
    { parent: self, deleteBeforeReplace: true }
  );

  const nlb = new oci.networkloadbalancer.NetworkLoadBalancer(
    "nlb",
    {
      compartmentId: compartment.id,
      displayName: "fn0-hq",
      subnetId: subnet.id,
    },
    { parent: self }
  );

  return {
    vcn,
    nlb,
    subnet,
  };
}

function setHqBinaryObject(
  self: pulumi.Resource,
  compartment: oci.identity.Compartment
) {
  const bucket = new oci.objectstorage.Bucket("app-bucket", {
    compartmentId: compartment.id,
    namespace: compartment.id.apply((compartmentId) =>
      oci.objectstorage
        .getNamespace({ compartmentId })
        .then((ns) => ns.namespace)
    ),
    name: "fn0-hq-binary-object",
    accessType: "NoPublicAccess",
  });

  const appObject = new oci.objectstorage.StorageObject(
    "app-binary",
    {
      bucket: bucket.name,
      namespace: bucket.namespace,
      object: "hq",
      source: new pulumi.asset.FileAsset(binaryPath),
      contentType: "application/octet-stream",
    },
    { dependsOn: [buildCommand] }
  ); // 중요: 빌드가 끝난 후 업로드

  return {
    vcn,
    nlb,
    subnet,
  };
}
