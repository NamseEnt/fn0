import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage the [default AWS VPC](http://docs.aws.amazon.com/AmazonVPC/latest/UserGuide/default-vpc.html)
 * in the current AWS Region.
 *
 * If you created your AWS account after 2013-12-04 you have a default VPC in each AWS Region.
 *
 * **This is an advanced resource** and has special caveats to be aware of when using it. Please read this document in its entirety before using this resource.
 *
 * The `aws.ec2.DefaultVpc` resource behaves differently from normal resources in that if a default VPC exists, this provider does not _create_ this resource, but instead "adopts" it into management.
 * If no default VPC exists, the provider creates a new default VPC, which leads to the implicit creation of [other resources](https://docs.aws.amazon.com/vpc/latest/userguide/default-vpc.html#default-vpc-components).
 * By default, `pulumi destroy` does not delete the default VPC but does remove the resource from the state.
 * Set the `forceDestroy` argument to `true` to delete the default VPC.
 *
 * ## Example Usage
 *
 * Basic usage with tags:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.ec2.DefaultVpc("default", {tags: {
 *     Name: "Default VPC",
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Default VPCs using the VPC `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/defaultVpc:DefaultVpc default vpc-a01106c2
 * ```
 */
export declare class DefaultVpc extends pulumi.CustomResource {
    /**
     * Get an existing DefaultVpc resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DefaultVpcState, opts?: pulumi.CustomResourceOptions): DefaultVpc;
    /**
     * Returns true if the given object is an instance of DefaultVpc.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DefaultVpc;
    readonly arn: pulumi.Output<string>;
    readonly assignGeneratedIpv6CidrBlock: pulumi.Output<boolean | undefined>;
    /**
     * The primary IPv4 CIDR block for the VPC
     */
    readonly cidrBlock: pulumi.Output<string>;
    readonly defaultNetworkAclId: pulumi.Output<string>;
    readonly defaultRouteTableId: pulumi.Output<string>;
    readonly defaultSecurityGroupId: pulumi.Output<string>;
    readonly dhcpOptionsId: pulumi.Output<string>;
    readonly enableDnsHostnames: pulumi.Output<boolean | undefined>;
    readonly enableDnsSupport: pulumi.Output<boolean | undefined>;
    readonly enableNetworkAddressUsageMetrics: pulumi.Output<boolean>;
    readonly existingDefaultVpc: pulumi.Output<boolean>;
    /**
     * Whether destroying the resource deletes the default VPC. Default: `false`
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * The allowed tenancy of instances launched into the VPC
     */
    readonly instanceTenancy: pulumi.Output<string>;
    readonly ipv6AssociationId: pulumi.Output<string>;
    readonly ipv6CidrBlock: pulumi.Output<string>;
    readonly ipv6CidrBlockNetworkBorderGroup: pulumi.Output<string>;
    readonly ipv6IpamPoolId: pulumi.Output<string | undefined>;
    readonly ipv6NetmaskLength: pulumi.Output<number | undefined>;
    readonly mainRouteTableId: pulumi.Output<string>;
    readonly ownerId: pulumi.Output<string>;
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a DefaultVpc resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DefaultVpcArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DefaultVpc resources.
 */
export interface DefaultVpcState {
    arn?: pulumi.Input<string>;
    assignGeneratedIpv6CidrBlock?: pulumi.Input<boolean>;
    /**
     * The primary IPv4 CIDR block for the VPC
     */
    cidrBlock?: pulumi.Input<string>;
    defaultNetworkAclId?: pulumi.Input<string>;
    defaultRouteTableId?: pulumi.Input<string>;
    defaultSecurityGroupId?: pulumi.Input<string>;
    dhcpOptionsId?: pulumi.Input<string>;
    enableDnsHostnames?: pulumi.Input<boolean>;
    enableDnsSupport?: pulumi.Input<boolean>;
    enableNetworkAddressUsageMetrics?: pulumi.Input<boolean>;
    existingDefaultVpc?: pulumi.Input<boolean>;
    /**
     * Whether destroying the resource deletes the default VPC. Default: `false`
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The allowed tenancy of instances launched into the VPC
     */
    instanceTenancy?: pulumi.Input<string>;
    ipv6AssociationId?: pulumi.Input<string>;
    ipv6CidrBlock?: pulumi.Input<string>;
    ipv6CidrBlockNetworkBorderGroup?: pulumi.Input<string>;
    ipv6IpamPoolId?: pulumi.Input<string>;
    ipv6NetmaskLength?: pulumi.Input<number>;
    mainRouteTableId?: pulumi.Input<string>;
    ownerId?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a DefaultVpc resource.
 */
export interface DefaultVpcArgs {
    assignGeneratedIpv6CidrBlock?: pulumi.Input<boolean>;
    enableDnsHostnames?: pulumi.Input<boolean>;
    enableDnsSupport?: pulumi.Input<boolean>;
    enableNetworkAddressUsageMetrics?: pulumi.Input<boolean>;
    /**
     * Whether destroying the resource deletes the default VPC. Default: `false`
     */
    forceDestroy?: pulumi.Input<boolean>;
    ipv6CidrBlock?: pulumi.Input<string>;
    ipv6CidrBlockNetworkBorderGroup?: pulumi.Input<string>;
    ipv6IpamPoolId?: pulumi.Input<string>;
    ipv6NetmaskLength?: pulumi.Input<number>;
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
