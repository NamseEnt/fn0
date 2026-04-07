import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to associate additional IPv6 CIDR blocks with a VPC.
 *
 * The `aws.ec2.VpcIpv6CidrBlockAssociation` resource allows IPv6 CIDR blocks to be added to the VPC.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ec2.Vpc("test", {cidrBlock: "10.0.0.0/16"});
 * const testVpcIpv6CidrBlockAssociation = new aws.ec2.VpcIpv6CidrBlockAssociation("test", {
 *     ipv6IpamPoolId: testAwsVpcIpamPool.id,
 *     vpcId: test.id,
 * });
 * ```
 *
 * ## Import
 *
 * or
 *
 * or
 *
 * Using `pulumi import`, import `aws_vpc_ipv6_cidr_block_association` using the VPC CIDR association ID and optionally the IPv6 IPAM pool ID and netmask length. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpv6CidrBlockAssociation:VpcIpv6CidrBlockAssociation example vpc-cidr-assoc-0754129087e149dcd
 * ```
 * or
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpv6CidrBlockAssociation:VpcIpv6CidrBlockAssociation example vpc-cidr-assoc-0754129087e149dcd,ipam-pool-0611d1d6bbc05ce60
 * ```
 * or
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpv6CidrBlockAssociation:VpcIpv6CidrBlockAssociation example vpc-cidr-assoc-0754129087e149dcd,ipam-pool-0611d1d6bbc05ce60,56
 * ```
 */
export declare class VpcIpv6CidrBlockAssociation extends pulumi.CustomResource {
    /**
     * Get an existing VpcIpv6CidrBlockAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcIpv6CidrBlockAssociationState, opts?: pulumi.CustomResourceOptions): VpcIpv6CidrBlockAssociation;
    /**
     * Returns true if the given object is an instance of VpcIpv6CidrBlockAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcIpv6CidrBlockAssociation;
    /**
     * Requests an Amazon-provided IPv6 CIDR block with a /56 prefix length for the VPC. You cannot specify the range of IPv6 addresses, or the size of the CIDR block. Default is `false`. Conflicts with `ipv6IpamPoolId`, `ipv6Pool`, `ipv6CidrBlock` and `ipv6NetmaskLength`.
     */
    readonly assignGeneratedIpv6CidrBlock: pulumi.Output<boolean>;
    /**
     * The source that allocated the IP address space. Values: `amazon`, `byoip`, `none`.
     */
    readonly ipSource: pulumi.Output<string>;
    /**
     * Public IPv6 addresses are those advertised on the internet from AWS. Private IP addresses are not and cannot be advertised on the internet from AWS. Values: `public`, `private`.
     */
    readonly ipv6AddressAttribute: pulumi.Output<string>;
    /**
     * The IPv6 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv6NetmaskLength`. This parameter is required if `ipv6NetmaskLength` is not set and the IPAM pool does not have `allocationDefaultNetmask` set. Conflicts with `assignGeneratedIpv6CidrBlock`.
     */
    readonly ipv6CidrBlock: pulumi.Output<string>;
    /**
     * The ID of an IPv6 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Conflict with `assignGeneratedIpv6CidrBlock` and `ipv6Pool`.
     */
    readonly ipv6IpamPoolId: pulumi.Output<string | undefined>;
    /**
     * The netmask length of the IPv6 CIDR you want to allocate to this VPC. Requires specifying a `ipv6IpamPoolId`. This parameter is optional if the IPAM pool has `allocationDefaultNetmask` set, otherwise it or `ipv6CidrBlock` are required. Conflicts with `ipv6CidrBlock`.
     */
    readonly ipv6NetmaskLength: pulumi.Output<number | undefined>;
    /**
     * The  ID of an IPv6 address pool from which to allocate the IPv6 CIDR block. Conflicts with `assignGeneratedIpv6CidrBlock` and `ipv6IpamPoolId`.
     */
    readonly ipv6Pool: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the VPC to make the association with.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a VpcIpv6CidrBlockAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcIpv6CidrBlockAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcIpv6CidrBlockAssociation resources.
 */
export interface VpcIpv6CidrBlockAssociationState {
    /**
     * Requests an Amazon-provided IPv6 CIDR block with a /56 prefix length for the VPC. You cannot specify the range of IPv6 addresses, or the size of the CIDR block. Default is `false`. Conflicts with `ipv6IpamPoolId`, `ipv6Pool`, `ipv6CidrBlock` and `ipv6NetmaskLength`.
     */
    assignGeneratedIpv6CidrBlock?: pulumi.Input<boolean>;
    /**
     * The source that allocated the IP address space. Values: `amazon`, `byoip`, `none`.
     */
    ipSource?: pulumi.Input<string>;
    /**
     * Public IPv6 addresses are those advertised on the internet from AWS. Private IP addresses are not and cannot be advertised on the internet from AWS. Values: `public`, `private`.
     */
    ipv6AddressAttribute?: pulumi.Input<string>;
    /**
     * The IPv6 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv6NetmaskLength`. This parameter is required if `ipv6NetmaskLength` is not set and the IPAM pool does not have `allocationDefaultNetmask` set. Conflicts with `assignGeneratedIpv6CidrBlock`.
     */
    ipv6CidrBlock?: pulumi.Input<string>;
    /**
     * The ID of an IPv6 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Conflict with `assignGeneratedIpv6CidrBlock` and `ipv6Pool`.
     */
    ipv6IpamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the IPv6 CIDR you want to allocate to this VPC. Requires specifying a `ipv6IpamPoolId`. This parameter is optional if the IPAM pool has `allocationDefaultNetmask` set, otherwise it or `ipv6CidrBlock` are required. Conflicts with `ipv6CidrBlock`.
     */
    ipv6NetmaskLength?: pulumi.Input<number>;
    /**
     * The  ID of an IPv6 address pool from which to allocate the IPv6 CIDR block. Conflicts with `assignGeneratedIpv6CidrBlock` and `ipv6IpamPoolId`.
     */
    ipv6Pool?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC to make the association with.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcIpv6CidrBlockAssociation resource.
 */
export interface VpcIpv6CidrBlockAssociationArgs {
    /**
     * Requests an Amazon-provided IPv6 CIDR block with a /56 prefix length for the VPC. You cannot specify the range of IPv6 addresses, or the size of the CIDR block. Default is `false`. Conflicts with `ipv6IpamPoolId`, `ipv6Pool`, `ipv6CidrBlock` and `ipv6NetmaskLength`.
     */
    assignGeneratedIpv6CidrBlock?: pulumi.Input<boolean>;
    /**
     * The IPv6 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv6NetmaskLength`. This parameter is required if `ipv6NetmaskLength` is not set and the IPAM pool does not have `allocationDefaultNetmask` set. Conflicts with `assignGeneratedIpv6CidrBlock`.
     */
    ipv6CidrBlock?: pulumi.Input<string>;
    /**
     * The ID of an IPv6 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Conflict with `assignGeneratedIpv6CidrBlock` and `ipv6Pool`.
     */
    ipv6IpamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the IPv6 CIDR you want to allocate to this VPC. Requires specifying a `ipv6IpamPoolId`. This parameter is optional if the IPAM pool has `allocationDefaultNetmask` set, otherwise it or `ipv6CidrBlock` are required. Conflicts with `ipv6CidrBlock`.
     */
    ipv6NetmaskLength?: pulumi.Input<number>;
    /**
     * The  ID of an IPv6 address pool from which to allocate the IPv6 CIDR block. Conflicts with `assignGeneratedIpv6CidrBlock` and `ipv6IpamPoolId`.
     */
    ipv6Pool?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC to make the association with.
     */
    vpcId: pulumi.Input<string>;
}
