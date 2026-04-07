import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to create a VPC NAT Gateway.
 *
 * !> **WARNING:** You should not use the `aws.ec2.NatGateway` resource that has `secondaryAllocationIds` in conjunction with an `aws.ec2.NatGatewayEipAssociation` resource. Doing so may cause perpetual differences, and result in associations being overwritten.
 *
 * ## Example Usage
 *
 * ### Public NAT
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.NatGateway("example", {
 *     allocationId: exampleAwsEip.id,
 *     subnetId: exampleAwsSubnet.id,
 *     tags: {
 *         Name: "gw NAT",
 *     },
 * }, {
 *     dependsOn: [exampleAwsInternetGateway],
 * });
 * ```
 *
 * ### Public NAT with Secondary Private IP Addresses
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.NatGateway("example", {
 *     allocationId: exampleAwsEip.id,
 *     subnetId: exampleAwsSubnet.id,
 *     secondaryAllocationIds: [secondary.id],
 *     secondaryPrivateIpAddresses: ["10.0.1.5"],
 * });
 * ```
 *
 * ### Private NAT
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.NatGateway("example", {
 *     connectivityType: "private",
 *     subnetId: exampleAwsSubnet.id,
 * });
 * ```
 *
 * ### Private NAT with Secondary Private IP Addresses
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.NatGateway("example", {
 *     connectivityType: "private",
 *     subnetId: exampleAwsSubnet.id,
 *     secondaryPrivateIpAddressCount: 7,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import NAT Gateways using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/natGateway:NatGateway private_gw nat-05dba92075d71c408
 * ```
 */
export declare class NatGateway extends pulumi.CustomResource {
    /**
     * Get an existing NatGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NatGatewayState, opts?: pulumi.CustomResourceOptions): NatGateway;
    /**
     * Returns true if the given object is an instance of NatGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NatGateway;
    /**
     * The Allocation ID of the Elastic IP address for the NAT Gateway. Required for `connectivityType` of `public`.
     */
    readonly allocationId: pulumi.Output<string | undefined>;
    /**
     * The association ID of the Elastic IP address that's associated with the NAT Gateway. Only available when `connectivityType` is `public`.
     */
    readonly associationId: pulumi.Output<string>;
    /**
     * Connectivity type for the NAT Gateway. Valid values are `private` and `public`. Defaults to `public`.
     */
    readonly connectivityType: pulumi.Output<string | undefined>;
    /**
     * The ID of the network interface associated with the NAT Gateway.
     */
    readonly networkInterfaceId: pulumi.Output<string>;
    /**
     * The private IPv4 address to assign to the NAT Gateway. If you don't provide an address, a private IPv4 address will be automatically assigned.
     */
    readonly privateIp: pulumi.Output<string>;
    /**
     * The Elastic IP address associated with the NAT Gateway.
     */
    readonly publicIp: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of secondary allocation EIP IDs for this NAT Gateway. To remove all secondary allocations an empty list should be specified.
     */
    readonly secondaryAllocationIds: pulumi.Output<string[]>;
    /**
     * [Private NAT Gateway only] The number of secondary private IPv4 addresses you want to assign to the NAT Gateway.
     */
    readonly secondaryPrivateIpAddressCount: pulumi.Output<number>;
    /**
     * A list of secondary private IPv4 addresses to assign to the NAT Gateway. To remove all secondary private addresses an empty list should be specified.
     */
    readonly secondaryPrivateIpAddresses: pulumi.Output<string[]>;
    /**
     * The Subnet ID of the subnet in which to place the NAT Gateway.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a NatGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NatGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NatGateway resources.
 */
export interface NatGatewayState {
    /**
     * The Allocation ID of the Elastic IP address for the NAT Gateway. Required for `connectivityType` of `public`.
     */
    allocationId?: pulumi.Input<string>;
    /**
     * The association ID of the Elastic IP address that's associated with the NAT Gateway. Only available when `connectivityType` is `public`.
     */
    associationId?: pulumi.Input<string>;
    /**
     * Connectivity type for the NAT Gateway. Valid values are `private` and `public`. Defaults to `public`.
     */
    connectivityType?: pulumi.Input<string>;
    /**
     * The ID of the network interface associated with the NAT Gateway.
     */
    networkInterfaceId?: pulumi.Input<string>;
    /**
     * The private IPv4 address to assign to the NAT Gateway. If you don't provide an address, a private IPv4 address will be automatically assigned.
     */
    privateIp?: pulumi.Input<string>;
    /**
     * The Elastic IP address associated with the NAT Gateway.
     */
    publicIp?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of secondary allocation EIP IDs for this NAT Gateway. To remove all secondary allocations an empty list should be specified.
     */
    secondaryAllocationIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * [Private NAT Gateway only] The number of secondary private IPv4 addresses you want to assign to the NAT Gateway.
     */
    secondaryPrivateIpAddressCount?: pulumi.Input<number>;
    /**
     * A list of secondary private IPv4 addresses to assign to the NAT Gateway. To remove all secondary private addresses an empty list should be specified.
     */
    secondaryPrivateIpAddresses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Subnet ID of the subnet in which to place the NAT Gateway.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a NatGateway resource.
 */
export interface NatGatewayArgs {
    /**
     * The Allocation ID of the Elastic IP address for the NAT Gateway. Required for `connectivityType` of `public`.
     */
    allocationId?: pulumi.Input<string>;
    /**
     * Connectivity type for the NAT Gateway. Valid values are `private` and `public`. Defaults to `public`.
     */
    connectivityType?: pulumi.Input<string>;
    /**
     * The private IPv4 address to assign to the NAT Gateway. If you don't provide an address, a private IPv4 address will be automatically assigned.
     */
    privateIp?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of secondary allocation EIP IDs for this NAT Gateway. To remove all secondary allocations an empty list should be specified.
     */
    secondaryAllocationIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * [Private NAT Gateway only] The number of secondary private IPv4 addresses you want to assign to the NAT Gateway.
     */
    secondaryPrivateIpAddressCount?: pulumi.Input<number>;
    /**
     * A list of secondary private IPv4 addresses to assign to the NAT Gateway. To remove all secondary private addresses an empty list should be specified.
     */
    secondaryPrivateIpAddresses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Subnet ID of the subnet in which to place the NAT Gateway.
     */
    subnetId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
