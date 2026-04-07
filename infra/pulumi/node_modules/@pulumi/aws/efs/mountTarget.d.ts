import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic File System (EFS) mount target.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.ec2.Vpc("foo", {cidrBlock: "10.0.0.0/16"});
 * const alphaSubnet = new aws.ec2.Subnet("alpha", {
 *     vpcId: foo.id,
 *     availabilityZone: "us-west-2a",
 *     cidrBlock: "10.0.1.0/24",
 * });
 * const alpha = new aws.efs.MountTarget("alpha", {
 *     fileSystemId: fooAwsEfsFileSystem.id,
 *     subnetId: alphaSubnet.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the EFS mount targets using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:efs/mountTarget:MountTarget alpha fsmt-52a643fb
 * ```
 */
export declare class MountTarget extends pulumi.CustomResource {
    /**
     * Get an existing MountTarget resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MountTargetState, opts?: pulumi.CustomResourceOptions): MountTarget;
    /**
     * Returns true if the given object is an instance of MountTarget.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MountTarget;
    /**
     * The unique and consistent identifier of the Availability Zone (AZ) that the mount target resides in.
     */
    readonly availabilityZoneId: pulumi.Output<string>;
    /**
     * The name of the Availability Zone (AZ) that the mount target resides in.
     */
    readonly availabilityZoneName: pulumi.Output<string>;
    /**
     * The DNS name for the EFS file system.
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * Amazon Resource Name of the file system.
     */
    readonly fileSystemArn: pulumi.Output<string>;
    /**
     * The ID of the file system for which the mount target is intended.
     */
    readonly fileSystemId: pulumi.Output<string>;
    /**
     * The address (within the address range of the specified subnet) at
     * which the file system may be mounted via the mount target.
     */
    readonly ipAddress: pulumi.Output<string>;
    /**
     * IP address type for the mount target. Valid values are `IPV4_ONLY` (only IPv4 addresses), `IPV6_ONLY` (only IPv6 addresses), and `DUAL_STACK` (dual-stack, both IPv4 and IPv6 addresses). Defaults to `IPV4_ONLY`.
     */
    readonly ipAddressType: pulumi.Output<string>;
    /**
     * IPv6 address to use. Valid only when `ipAddressType` is set to `IPV6_ONLY` or `DUAL_STACK`.
     */
    readonly ipv6Address: pulumi.Output<string>;
    /**
     * The DNS name for the given subnet/AZ per [documented convention](http://docs.aws.amazon.com/efs/latest/ug/mounting-fs-mount-cmd-dns-name.html).
     */
    readonly mountTargetDnsName: pulumi.Output<string>;
    /**
     * The ID of the network interface that Amazon EFS created when it created the mount target.
     */
    readonly networkInterfaceId: pulumi.Output<string>;
    /**
     * AWS account ID that owns the resource.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of up to 5 VPC security group IDs (that must
     * be for the same VPC as subnet specified) in effect for the mount target.
     */
    readonly securityGroups: pulumi.Output<string[]>;
    /**
     * The ID of the subnet to add the mount target in.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * Create a MountTarget resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MountTargetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MountTarget resources.
 */
export interface MountTargetState {
    /**
     * The unique and consistent identifier of the Availability Zone (AZ) that the mount target resides in.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The name of the Availability Zone (AZ) that the mount target resides in.
     */
    availabilityZoneName?: pulumi.Input<string>;
    /**
     * The DNS name for the EFS file system.
     */
    dnsName?: pulumi.Input<string>;
    /**
     * Amazon Resource Name of the file system.
     */
    fileSystemArn?: pulumi.Input<string>;
    /**
     * The ID of the file system for which the mount target is intended.
     */
    fileSystemId?: pulumi.Input<string>;
    /**
     * The address (within the address range of the specified subnet) at
     * which the file system may be mounted via the mount target.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * IP address type for the mount target. Valid values are `IPV4_ONLY` (only IPv4 addresses), `IPV6_ONLY` (only IPv6 addresses), and `DUAL_STACK` (dual-stack, both IPv4 and IPv6 addresses). Defaults to `IPV4_ONLY`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * IPv6 address to use. Valid only when `ipAddressType` is set to `IPV6_ONLY` or `DUAL_STACK`.
     */
    ipv6Address?: pulumi.Input<string>;
    /**
     * The DNS name for the given subnet/AZ per [documented convention](http://docs.aws.amazon.com/efs/latest/ug/mounting-fs-mount-cmd-dns-name.html).
     */
    mountTargetDnsName?: pulumi.Input<string>;
    /**
     * The ID of the network interface that Amazon EFS created when it created the mount target.
     */
    networkInterfaceId?: pulumi.Input<string>;
    /**
     * AWS account ID that owns the resource.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of up to 5 VPC security group IDs (that must
     * be for the same VPC as subnet specified) in effect for the mount target.
     */
    securityGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the subnet to add the mount target in.
     */
    subnetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MountTarget resource.
 */
export interface MountTargetArgs {
    /**
     * The ID of the file system for which the mount target is intended.
     */
    fileSystemId: pulumi.Input<string>;
    /**
     * The address (within the address range of the specified subnet) at
     * which the file system may be mounted via the mount target.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * IP address type for the mount target. Valid values are `IPV4_ONLY` (only IPv4 addresses), `IPV6_ONLY` (only IPv6 addresses), and `DUAL_STACK` (dual-stack, both IPv4 and IPv6 addresses). Defaults to `IPV4_ONLY`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * IPv6 address to use. Valid only when `ipAddressType` is set to `IPV6_ONLY` or `DUAL_STACK`.
     */
    ipv6Address?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of up to 5 VPC security group IDs (that must
     * be for the same VPC as subnet specified) in effect for the mount target.
     */
    securityGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the subnet to add the mount target in.
     */
    subnetId: pulumi.Input<string>;
}
