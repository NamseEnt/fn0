import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a VPC resource.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.ec2.Vpc("main", {cidrBlock: "10.0.0.0/16"});
 * ```
 *
 * Basic usage with tags:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.ec2.Vpc("main", {
 *     cidrBlock: "10.0.0.0/16",
 *     instanceTenancy: "default",
 *     tags: {
 *         Name: "main",
 *     },
 * });
 * ```
 *
 * VPC with CIDR from AWS IPAM:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const test = new aws.ec2.VpcIpam("test", {operatingRegions: [{
 *     regionName: current.then(current => current.region),
 * }]});
 * const testVpcIpamPool = new aws.ec2.VpcIpamPool("test", {
 *     addressFamily: "ipv4",
 *     ipamScopeId: test.privateDefaultScopeId,
 *     locale: current.then(current => current.region),
 * });
 * const testVpcIpamPoolCidr = new aws.ec2.VpcIpamPoolCidr("test", {
 *     ipamPoolId: testVpcIpamPool.id,
 *     cidr: "172.20.0.0/16",
 * });
 * const testVpc = new aws.ec2.Vpc("test", {
 *     ipv4IpamPoolId: testVpcIpamPool.id,
 *     ipv4NetmaskLength: 28,
 * }, {
 *     dependsOn: [testVpcIpamPoolCidr],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPCs using the VPC `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpc:Vpc test_vpc vpc-a01106c2
 * ```
 */
export declare class Vpc extends pulumi.CustomResource {
    /**
     * Get an existing Vpc resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcState, opts?: pulumi.CustomResourceOptions): Vpc;
    /**
     * Returns true if the given object is an instance of Vpc.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Vpc;
    /**
     * Amazon Resource Name (ARN) of VPC
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Requests an Amazon-provided IPv6 CIDR block with a /56 prefix length for the VPC. You cannot specify the range of IP addresses, or the size of the CIDR block. Default is `false`. Conflicts with `ipv6IpamPoolId`
     */
    readonly assignGeneratedIpv6CidrBlock: pulumi.Output<boolean | undefined>;
    /**
     * The IPv4 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv4NetmaskLength`.
     */
    readonly cidrBlock: pulumi.Output<string>;
    /**
     * The ID of the network ACL created by default on VPC creation
     */
    readonly defaultNetworkAclId: pulumi.Output<string>;
    /**
     * The ID of the route table created by default on VPC creation
     */
    readonly defaultRouteTableId: pulumi.Output<string>;
    /**
     * The ID of the security group created by default on VPC creation
     */
    readonly defaultSecurityGroupId: pulumi.Output<string>;
    /**
     * DHCP options id of the desired VPC.
     */
    readonly dhcpOptionsId: pulumi.Output<string>;
    /**
     * A boolean flag to enable/disable DNS hostnames in the VPC. Defaults false.
     */
    readonly enableDnsHostnames: pulumi.Output<boolean>;
    /**
     * A boolean flag to enable/disable DNS support in the VPC. Defaults to true.
     */
    readonly enableDnsSupport: pulumi.Output<boolean | undefined>;
    /**
     * Indicates whether Network Address Usage metrics are enabled for your VPC. Defaults to false.
     */
    readonly enableNetworkAddressUsageMetrics: pulumi.Output<boolean>;
    /**
     * A tenancy option for instances launched into the VPC. Default is `default`, which ensures that EC2 instances launched in this VPC use the EC2 instance tenancy attribute specified when the EC2 instance is launched. The only other option is `dedicated`, which ensures that EC2 instances launched in this VPC are run on dedicated tenancy instances regardless of the tenancy attribute specified at launch. This has a dedicated per region fee of $2 per hour, plus an hourly per instance usage fee.
     */
    readonly instanceTenancy: pulumi.Output<string | undefined>;
    /**
     * The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Using IPAM you can monitor IP address usage throughout your AWS Organization.
     */
    readonly ipv4IpamPoolId: pulumi.Output<string | undefined>;
    /**
     * The netmask length of the IPv4 CIDR you want to allocate to this VPC. Requires specifying a `ipv4IpamPoolId`.
     */
    readonly ipv4NetmaskLength: pulumi.Output<number | undefined>;
    /**
     * The association ID for the IPv6 CIDR block.
     */
    readonly ipv6AssociationId: pulumi.Output<string>;
    /**
     * IPv6 CIDR block to request from an IPAM Pool. Can be set explicitly or derived from IPAM using `ipv6NetmaskLength`.
     */
    readonly ipv6CidrBlock: pulumi.Output<string>;
    /**
     * By default when an IPv6 CIDR is assigned to a VPC a default ipv6CidrBlockNetworkBorderGroup will be set to the region of the VPC. This can be changed to restrict advertisement of public addresses to specific Network Border Groups such as LocalZones.
     */
    readonly ipv6CidrBlockNetworkBorderGroup: pulumi.Output<string>;
    /**
     * IPAM Pool ID for a IPv6 pool. Conflicts with `assignGeneratedIpv6CidrBlock`.
     */
    readonly ipv6IpamPoolId: pulumi.Output<string | undefined>;
    /**
     * Netmask length to request from IPAM Pool. Conflicts with `ipv6CidrBlock`. This can be omitted if IPAM pool as a `allocationDefaultNetmaskLength` set. Valid values are from `44` to `60` in increments of 4.
     */
    readonly ipv6NetmaskLength: pulumi.Output<number | undefined>;
    /**
     * The ID of the main route table associated with
     * this VPC. Note that you can change a VPC's main route table by using an
     * `aws.ec2.MainRouteTableAssociation`.
     */
    readonly mainRouteTableId: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the VPC.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Vpc resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: VpcArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Vpc resources.
 */
export interface VpcState {
    /**
     * Amazon Resource Name (ARN) of VPC
     */
    arn?: pulumi.Input<string>;
    /**
     * Requests an Amazon-provided IPv6 CIDR block with a /56 prefix length for the VPC. You cannot specify the range of IP addresses, or the size of the CIDR block. Default is `false`. Conflicts with `ipv6IpamPoolId`
     */
    assignGeneratedIpv6CidrBlock?: pulumi.Input<boolean>;
    /**
     * The IPv4 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv4NetmaskLength`.
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * The ID of the network ACL created by default on VPC creation
     */
    defaultNetworkAclId?: pulumi.Input<string>;
    /**
     * The ID of the route table created by default on VPC creation
     */
    defaultRouteTableId?: pulumi.Input<string>;
    /**
     * The ID of the security group created by default on VPC creation
     */
    defaultSecurityGroupId?: pulumi.Input<string>;
    /**
     * DHCP options id of the desired VPC.
     */
    dhcpOptionsId?: pulumi.Input<string>;
    /**
     * A boolean flag to enable/disable DNS hostnames in the VPC. Defaults false.
     */
    enableDnsHostnames?: pulumi.Input<boolean>;
    /**
     * A boolean flag to enable/disable DNS support in the VPC. Defaults to true.
     */
    enableDnsSupport?: pulumi.Input<boolean>;
    /**
     * Indicates whether Network Address Usage metrics are enabled for your VPC. Defaults to false.
     */
    enableNetworkAddressUsageMetrics?: pulumi.Input<boolean>;
    /**
     * A tenancy option for instances launched into the VPC. Default is `default`, which ensures that EC2 instances launched in this VPC use the EC2 instance tenancy attribute specified when the EC2 instance is launched. The only other option is `dedicated`, which ensures that EC2 instances launched in this VPC are run on dedicated tenancy instances regardless of the tenancy attribute specified at launch. This has a dedicated per region fee of $2 per hour, plus an hourly per instance usage fee.
     */
    instanceTenancy?: pulumi.Input<string>;
    /**
     * The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Using IPAM you can monitor IP address usage throughout your AWS Organization.
     */
    ipv4IpamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the IPv4 CIDR you want to allocate to this VPC. Requires specifying a `ipv4IpamPoolId`.
     */
    ipv4NetmaskLength?: pulumi.Input<number>;
    /**
     * The association ID for the IPv6 CIDR block.
     */
    ipv6AssociationId?: pulumi.Input<string>;
    /**
     * IPv6 CIDR block to request from an IPAM Pool. Can be set explicitly or derived from IPAM using `ipv6NetmaskLength`.
     */
    ipv6CidrBlock?: pulumi.Input<string>;
    /**
     * By default when an IPv6 CIDR is assigned to a VPC a default ipv6CidrBlockNetworkBorderGroup will be set to the region of the VPC. This can be changed to restrict advertisement of public addresses to specific Network Border Groups such as LocalZones.
     */
    ipv6CidrBlockNetworkBorderGroup?: pulumi.Input<string>;
    /**
     * IPAM Pool ID for a IPv6 pool. Conflicts with `assignGeneratedIpv6CidrBlock`.
     */
    ipv6IpamPoolId?: pulumi.Input<string>;
    /**
     * Netmask length to request from IPAM Pool. Conflicts with `ipv6CidrBlock`. This can be omitted if IPAM pool as a `allocationDefaultNetmaskLength` set. Valid values are from `44` to `60` in increments of 4.
     */
    ipv6NetmaskLength?: pulumi.Input<number>;
    /**
     * The ID of the main route table associated with
     * this VPC. Note that you can change a VPC's main route table by using an
     * `aws.ec2.MainRouteTableAssociation`.
     */
    mainRouteTableId?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the VPC.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Vpc resource.
 */
export interface VpcArgs {
    /**
     * Requests an Amazon-provided IPv6 CIDR block with a /56 prefix length for the VPC. You cannot specify the range of IP addresses, or the size of the CIDR block. Default is `false`. Conflicts with `ipv6IpamPoolId`
     */
    assignGeneratedIpv6CidrBlock?: pulumi.Input<boolean>;
    /**
     * The IPv4 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv4NetmaskLength`.
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * A boolean flag to enable/disable DNS hostnames in the VPC. Defaults false.
     */
    enableDnsHostnames?: pulumi.Input<boolean>;
    /**
     * A boolean flag to enable/disable DNS support in the VPC. Defaults to true.
     */
    enableDnsSupport?: pulumi.Input<boolean>;
    /**
     * Indicates whether Network Address Usage metrics are enabled for your VPC. Defaults to false.
     */
    enableNetworkAddressUsageMetrics?: pulumi.Input<boolean>;
    /**
     * A tenancy option for instances launched into the VPC. Default is `default`, which ensures that EC2 instances launched in this VPC use the EC2 instance tenancy attribute specified when the EC2 instance is launched. The only other option is `dedicated`, which ensures that EC2 instances launched in this VPC are run on dedicated tenancy instances regardless of the tenancy attribute specified at launch. This has a dedicated per region fee of $2 per hour, plus an hourly per instance usage fee.
     */
    instanceTenancy?: pulumi.Input<string>;
    /**
     * The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Using IPAM you can monitor IP address usage throughout your AWS Organization.
     */
    ipv4IpamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the IPv4 CIDR you want to allocate to this VPC. Requires specifying a `ipv4IpamPoolId`.
     */
    ipv4NetmaskLength?: pulumi.Input<number>;
    /**
     * IPv6 CIDR block to request from an IPAM Pool. Can be set explicitly or derived from IPAM using `ipv6NetmaskLength`.
     */
    ipv6CidrBlock?: pulumi.Input<string>;
    /**
     * By default when an IPv6 CIDR is assigned to a VPC a default ipv6CidrBlockNetworkBorderGroup will be set to the region of the VPC. This can be changed to restrict advertisement of public addresses to specific Network Border Groups such as LocalZones.
     */
    ipv6CidrBlockNetworkBorderGroup?: pulumi.Input<string>;
    /**
     * IPAM Pool ID for a IPv6 pool. Conflicts with `assignGeneratedIpv6CidrBlock`.
     */
    ipv6IpamPoolId?: pulumi.Input<string>;
    /**
     * Netmask length to request from IPAM Pool. Conflicts with `ipv6CidrBlock`. This can be omitted if IPAM pool as a `allocationDefaultNetmaskLength` set. Valid values are from `44` to `60` in increments of 4.
     */
    ipv6NetmaskLength?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
