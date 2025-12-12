import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import { Address6 } from "ip-address";
import * as command from "@pulumi/command";

export function createNetworking(
  parent: pulumi.Resource,
  {
    compartmentId,
    vcnId,
    ipv6cidrBlocks,
  }: {
    compartmentId: pulumi.Input<string>;
    vcnId: pulumi.Input<string>;
    ipv6cidrBlocks: pulumi.Input<string[]>;
  }
): {
  regionalSubnet: oci.core.Subnet;
} {
  const internetGateway = new oci.core.InternetGateway(
    "igw",
    {
      compartmentId,
      vcnId,
    },
    { parent }
  );

  const routeTable = new oci.core.RouteTable(
    "route-table",
    {
      compartmentId,
      vcnId,
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
    { parent }
  );

  const myIp = command.local.runOutput({
    command: "curl -s ifconfig.co",
  }).stdout;

  const ipv6cidrBlock = pulumi.output(ipv6cidrBlocks).apply((blocks) => {
    const cidr = blocks[0];
    const address = new Address6(cidr);
    const startAddress = address.startAddress();
    return `${startAddress.correctForm()}/64`;
  });

  const securityList = new oci.core.SecurityList(
    "security-list",
    {
      compartmentId,
      vcnId,
      egressSecurityRules: [
        {
          destination: "0.0.0.0/0",
          destinationType: "CIDR_BLOCK",
          protocol: "all",
          stateless: false,
        },
        {
          destination: "::/0",
          destinationType: "CIDR_BLOCK",
          protocol: "all",
          stateless: false,
        },
      ],
      ingressSecurityRules: [
        {
          source: myIp.apply((ip) => `${ip}/32`),
          protocol: "all",
          stateless: false,
        },
        {
          source: "10.0.0.0/16",
          protocol: "all",
          stateless: false,
        },
        {
          source: ipv6cidrBlock,
          protocol: "all",
          stateless: false,
        },
      ],
    },
    { parent }
  );

  const regionalSubnet = new oci.core.Subnet(
    "regional-subnet",
    {
      displayName: "fn0-hq-regional-subnet",
      compartmentId,
      vcnId,
      ipv4cidrBlocks: ["10.0.2.0/24"],
      ipv6cidrBlock,
      prohibitPublicIpOnVnic: false,
      routeTableId: routeTable.id,
      securityListIds: [securityList.id],
    },
    { parent, deleteBeforeReplace: true }
  );

  return { regionalSubnet };
}
