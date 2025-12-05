import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import { Subnet, SecurityGroup, Vpc } from "@pulumi/aws/ec2";

export interface AwsWatchdogVpcArgs {
  region: pulumi.Input<string>;
}

export class AwsWatchdogVpc extends pulumi.ComponentResource {
  public readonly vpc: Vpc;
  public readonly subnet: Subnet;
  public readonly securityGroup: SecurityGroup;
  public readonly ipv6CiderBlock: pulumi.Output<string>;

  constructor(
    name: string,
    args: AwsWatchdogVpcArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:aws-watchdog-waker-vpc", name, args, opts);

    const { region } = args;

    const vpc = new aws.ec2.Vpc("ipv6-vpc", {
      region,
      assignGeneratedIpv6CidrBlock: true,
      enableDnsHostnames: true,
    });
    this.vpc = vpc;
    this.ipv6CiderBlock = vpc.ipv6CidrBlock;

    const eigw = new aws.ec2.EgressOnlyInternetGateway("ipv6-eigw", {
      region,
      vpcId: vpc.id,
    });

    const subnet = new aws.ec2.Subnet("ipv6-native-subnet", {
      vpcId: vpc.id,
      ipv6CidrBlock: vpc.ipv6CidrBlock.apply((cidr) => {
        if (!cidr) return "";
        const prefix = cidr.split("::/")[0];
        return `${prefix}00::/64`;
      }),
      assignIpv6AddressOnCreation: true,
      ipv6Native: true,
      mapPublicIpOnLaunch: false,
    });
    this.subnet = subnet;

    const routeTable = new aws.ec2.RouteTable("ipv6-rt", {
      region,
      vpcId: vpc.id,
    });

    new aws.ec2.Route("ipv6-route", {
      region,
      routeTableId: routeTable.id,
      destinationIpv6CidrBlock: "::/0",
      egressOnlyGatewayId: eigw.id,
    });

    new aws.ec2.RouteTableAssociation("rt-assoc", {
      region,
      subnetId: subnet.id,
      routeTableId: routeTable.id,
    });

    const securityGroup = new aws.ec2.SecurityGroup("ipv6-lambda-sg", {
      region,
      vpcId: vpc.id,
      description: "Allow IPv6 traffic only",
      egress: [
        {
          protocol: "-1",
          fromPort: 0,
          toPort: 0,
          ipv6CidrBlocks: ["::/0"],
        },
      ],
    });
    this.securityGroup = securityGroup;
  }
}
