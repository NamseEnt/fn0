import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage a [default subnet](http://docs.aws.amazon.com/AmazonVPC/latest/UserGuide/default-vpc.html#default-vpc-basics) in the current region.
 *
 * **This is an advanced resource** and has special caveats to be aware of when using it. Please read this document in its entirety before using this resource.
 *
 * The `aws.ec2.DefaultSubnet` resource behaves differently from normal resources in that if a default subnet exists in the specified Availability Zone, this provider does not _create_ this resource, but instead "adopts" it into management.
 * If no default subnet exists, this provider creates a new default subnet.
 * By default, `pulumi destroy` does not delete the default subnet but does remove the resource from the state.
 * Set the `forceDestroy` argument to `true` to delete the default subnet.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const defaultAz1 = new aws.ec2.DefaultSubnet("default_az1", {
 *     availabilityZone: "us-west-2a",
 *     tags: {
 *         Name: "Default subnet for us-west-2a",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import subnets using the subnet `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/defaultSubnet:DefaultSubnet public_subnet subnet-9d4a7b6c
 * ```
 */
export declare class DefaultSubnet extends pulumi.CustomResource {
    /**
     * Get an existing DefaultSubnet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DefaultSubnetState, opts?: pulumi.CustomResourceOptions): DefaultSubnet;
    /**
     * Returns true if the given object is an instance of DefaultSubnet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DefaultSubnet;
    readonly arn: pulumi.Output<string>;
    readonly assignIpv6AddressOnCreation: pulumi.Output<boolean | undefined>;
    /**
     * is required
     * * The `availabilityZoneId`, `cidrBlock` and `vpcId` arguments become computed attributes
     * * The default value for `mapPublicIpOnLaunch` is `true`
     *
     * This resource supports the following additional arguments:
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * The AZ ID of the subnet
     */
    readonly availabilityZoneId: pulumi.Output<string>;
    /**
     * The IPv4 CIDR block assigned to the subnet
     */
    readonly cidrBlock: pulumi.Output<string>;
    readonly customerOwnedIpv4Pool: pulumi.Output<string | undefined>;
    readonly enableDns64: pulumi.Output<boolean | undefined>;
    readonly enableLniAtDeviceIndex: pulumi.Output<number>;
    readonly enableResourceNameDnsARecordOnLaunch: pulumi.Output<boolean | undefined>;
    readonly enableResourceNameDnsAaaaRecordOnLaunch: pulumi.Output<boolean | undefined>;
    readonly existingDefaultSubnet: pulumi.Output<boolean>;
    /**
     * Whether destroying the resource deletes the default subnet. Default: `false`
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    readonly ipv6CidrBlock: pulumi.Output<string>;
    readonly ipv6CidrBlockAssociationId: pulumi.Output<string>;
    readonly ipv6Native: pulumi.Output<boolean | undefined>;
    readonly mapCustomerOwnedIpOnLaunch: pulumi.Output<boolean | undefined>;
    readonly mapPublicIpOnLaunch: pulumi.Output<boolean | undefined>;
    readonly outpostArn: pulumi.Output<string>;
    readonly ownerId: pulumi.Output<string>;
    readonly privateDnsHostnameTypeOnLaunch: pulumi.Output<string>;
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ID of the VPC the subnet is in
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a DefaultSubnet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DefaultSubnetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DefaultSubnet resources.
 */
export interface DefaultSubnetState {
    arn?: pulumi.Input<string>;
    assignIpv6AddressOnCreation?: pulumi.Input<boolean>;
    /**
     * is required
     * * The `availabilityZoneId`, `cidrBlock` and `vpcId` arguments become computed attributes
     * * The default value for `mapPublicIpOnLaunch` is `true`
     *
     * This resource supports the following additional arguments:
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The AZ ID of the subnet
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The IPv4 CIDR block assigned to the subnet
     */
    cidrBlock?: pulumi.Input<string>;
    customerOwnedIpv4Pool?: pulumi.Input<string>;
    enableDns64?: pulumi.Input<boolean>;
    enableLniAtDeviceIndex?: pulumi.Input<number>;
    enableResourceNameDnsARecordOnLaunch?: pulumi.Input<boolean>;
    enableResourceNameDnsAaaaRecordOnLaunch?: pulumi.Input<boolean>;
    existingDefaultSubnet?: pulumi.Input<boolean>;
    /**
     * Whether destroying the resource deletes the default subnet. Default: `false`
     */
    forceDestroy?: pulumi.Input<boolean>;
    ipv6CidrBlock?: pulumi.Input<string>;
    ipv6CidrBlockAssociationId?: pulumi.Input<string>;
    ipv6Native?: pulumi.Input<boolean>;
    mapCustomerOwnedIpOnLaunch?: pulumi.Input<boolean>;
    mapPublicIpOnLaunch?: pulumi.Input<boolean>;
    outpostArn?: pulumi.Input<string>;
    ownerId?: pulumi.Input<string>;
    privateDnsHostnameTypeOnLaunch?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the VPC the subnet is in
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DefaultSubnet resource.
 */
export interface DefaultSubnetArgs {
    assignIpv6AddressOnCreation?: pulumi.Input<boolean>;
    /**
     * is required
     * * The `availabilityZoneId`, `cidrBlock` and `vpcId` arguments become computed attributes
     * * The default value for `mapPublicIpOnLaunch` is `true`
     *
     * This resource supports the following additional arguments:
     */
    availabilityZone: pulumi.Input<string>;
    customerOwnedIpv4Pool?: pulumi.Input<string>;
    enableDns64?: pulumi.Input<boolean>;
    enableResourceNameDnsARecordOnLaunch?: pulumi.Input<boolean>;
    enableResourceNameDnsAaaaRecordOnLaunch?: pulumi.Input<boolean>;
    /**
     * Whether destroying the resource deletes the default subnet. Default: `false`
     */
    forceDestroy?: pulumi.Input<boolean>;
    ipv6CidrBlock?: pulumi.Input<string>;
    ipv6Native?: pulumi.Input<boolean>;
    mapCustomerOwnedIpOnLaunch?: pulumi.Input<boolean>;
    mapPublicIpOnLaunch?: pulumi.Input<boolean>;
    privateDnsHostnameTypeOnLaunch?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
