import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to associate additional IPv4 CIDR blocks with a VPC.
 *
 * When a VPC is created, a primary IPv4 CIDR block for the VPC must be specified.
 * The `aws.ec2.VpcIpv4CidrBlockAssociation` resource allows further IPv4 CIDR blocks to be added to the VPC.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.ec2.Vpc("main", {cidrBlock: "10.0.0.0/16"});
 * const secondaryCidr = new aws.ec2.VpcIpv4CidrBlockAssociation("secondary_cidr", {
 *     vpcId: main.id,
 *     cidrBlock: "172.20.0.0/16",
 * });
 * ```
 *
 * ## Import
 *
 * or
 *
 * or
 *
 * Using `pulumi import`, import `aws_vpc_ipv4_cidr_block_association` using the VPC CIDR association ID and optionally the IPv4 IPAM pool ID and netmask length. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpv4CidrBlockAssociation:VpcIpv4CidrBlockAssociation example vpc-cidr-assoc-021e8461d70ed08be
 * ```
 * or
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpv4CidrBlockAssociation:VpcIpv4CidrBlockAssociation example vpc-cidr-assoc-021e8461d70ed08be,ipam-pool-0a07c432810393463
 * ```
 * or
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpv4CidrBlockAssociation:VpcIpv4CidrBlockAssociation example vpc-cidr-assoc-021e8461d70ed08be,ipam-pool-0a07c432810393463,28
 * ```
 */
export declare class VpcIpv4CidrBlockAssociation extends pulumi.CustomResource {
    /**
     * Get an existing VpcIpv4CidrBlockAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcIpv4CidrBlockAssociationState, opts?: pulumi.CustomResourceOptions): VpcIpv4CidrBlockAssociation;
    /**
     * Returns true if the given object is an instance of VpcIpv4CidrBlockAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcIpv4CidrBlockAssociation;
    /**
     * The IPv4 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv4NetmaskLength`.
     */
    readonly cidrBlock: pulumi.Output<string>;
    /**
     * The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Using IPAM you can monitor IP address usage throughout your AWS Organization.
     */
    readonly ipv4IpamPoolId: pulumi.Output<string | undefined>;
    /**
     * The netmask length of the IPv4 CIDR you want to allocate to this VPC. Requires specifying a `ipv4IpamPoolId`.
     */
    readonly ipv4NetmaskLength: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the VPC to make the association with.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a VpcIpv4CidrBlockAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcIpv4CidrBlockAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcIpv4CidrBlockAssociation resources.
 */
export interface VpcIpv4CidrBlockAssociationState {
    /**
     * The IPv4 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv4NetmaskLength`.
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Using IPAM you can monitor IP address usage throughout your AWS Organization.
     */
    ipv4IpamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the IPv4 CIDR you want to allocate to this VPC. Requires specifying a `ipv4IpamPoolId`.
     */
    ipv4NetmaskLength?: pulumi.Input<number>;
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
 * The set of arguments for constructing a VpcIpv4CidrBlockAssociation resource.
 */
export interface VpcIpv4CidrBlockAssociationArgs {
    /**
     * The IPv4 CIDR block for the VPC. CIDR can be explicitly set or it can be derived from IPAM using `ipv4NetmaskLength`.
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. IPAM is a VPC feature that you can use to automate your IP address management workflows including assigning, tracking, troubleshooting, and auditing IP addresses across AWS Regions and accounts. Using IPAM you can monitor IP address usage throughout your AWS Organization.
     */
    ipv4IpamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the IPv4 CIDR you want to allocate to this VPC. Requires specifying a `ipv4IpamPoolId`.
     */
    ipv4NetmaskLength?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC to make the association with.
     */
    vpcId: pulumi.Input<string>;
}
