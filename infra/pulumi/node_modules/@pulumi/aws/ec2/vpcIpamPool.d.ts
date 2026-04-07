import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an IP address pool resource for IPAM.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const example = new aws.ec2.VpcIpam("example", {operatingRegions: [{
 *     regionName: current.then(current => current.region),
 * }]});
 * const exampleVpcIpamPool = new aws.ec2.VpcIpamPool("example", {
 *     addressFamily: "ipv4",
 *     ipamScopeId: example.privateDefaultScopeId,
 *     locale: current.then(current => current.region),
 * });
 * ```
 *
 * Nested Pools:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const example = new aws.ec2.VpcIpam("example", {operatingRegions: [{
 *     regionName: current.then(current => current.region),
 * }]});
 * const parent = new aws.ec2.VpcIpamPool("parent", {
 *     addressFamily: "ipv4",
 *     ipamScopeId: example.privateDefaultScopeId,
 * });
 * const parentTest = new aws.ec2.VpcIpamPoolCidr("parent_test", {
 *     ipamPoolId: parent.id,
 *     cidr: "172.20.0.0/16",
 * });
 * const child = new aws.ec2.VpcIpamPool("child", {
 *     addressFamily: "ipv4",
 *     ipamScopeId: example.privateDefaultScopeId,
 *     locale: current.then(current => current.region),
 *     sourceIpamPoolId: parent.id,
 * });
 * const childTest = new aws.ec2.VpcIpamPoolCidr("child_test", {
 *     ipamPoolId: child.id,
 *     cidr: "172.20.0.0/24",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IPAMs using the IPAM pool `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpamPool:VpcIpamPool example ipam-pool-0958f95207d978e1e
 * ```
 */
export declare class VpcIpamPool extends pulumi.CustomResource {
    /**
     * Get an existing VpcIpamPool resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcIpamPoolState, opts?: pulumi.CustomResourceOptions): VpcIpamPool;
    /**
     * Returns true if the given object is an instance of VpcIpamPool.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcIpamPool;
    /**
     * The IP protocol assigned to this pool. You must choose either IPv4 or IPv6 protocol for a pool.
     */
    readonly addressFamily: pulumi.Output<string>;
    /**
     * A default netmask length for allocations added to this pool. If, for example, the CIDR assigned to this pool is 10.0.0.0/8 and you enter 16 here, new allocations will default to 10.0.0.0/16 (unless you provide a different netmask value when you create the new allocation).
     */
    readonly allocationDefaultNetmaskLength: pulumi.Output<number | undefined>;
    /**
     * The maximum netmask length that will be required for CIDR allocations in this pool.
     */
    readonly allocationMaxNetmaskLength: pulumi.Output<number | undefined>;
    /**
     * The minimum netmask length that will be required for CIDR allocations in this pool.
     */
    readonly allocationMinNetmaskLength: pulumi.Output<number | undefined>;
    /**
     * Tags that are required for resources that use CIDRs from this IPAM pool. Resources that do not have these tags will not be allowed to allocate space from the pool. If the resources have their tags changed after they have allocated space or if the allocation tagging requirements are changed on the pool, the resource may be marked as noncompliant.
     */
    readonly allocationResourceTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Amazon Resource Name (ARN) of IPAM
     */
    readonly arn: pulumi.Output<string>;
    /**
     * If you include this argument, IPAM automatically imports any VPCs you have in your scope that fall
     * within the CIDR range in the pool.
     */
    readonly autoImport: pulumi.Output<boolean | undefined>;
    /**
     * Limits which AWS service the pool can be used in. Only useable on public scopes. Valid Values: `ec2`.
     */
    readonly awsService: pulumi.Output<string | undefined>;
    /**
     * Enables you to quickly delete an IPAM pool and all resources within that pool, including provisioned CIDRs, allocations, and other pools.
     */
    readonly cascade: pulumi.Output<boolean | undefined>;
    /**
     * A description for the IPAM pool.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ID of the scope in which you would like to create the IPAM pool.
     */
    readonly ipamScopeId: pulumi.Output<string>;
    readonly ipamScopeType: pulumi.Output<string>;
    /**
     * The locale in which you would like to create the IPAM pool. Locale is the Region where you want to make an IPAM pool available for allocations. You can only create pools with locales that match the operating Regions of the IPAM. You can only create VPCs from a pool whose locale matches the VPC's Region. Possible values: Any AWS region, such as `us-east-1`.
     */
    readonly locale: pulumi.Output<string | undefined>;
    readonly poolDepth: pulumi.Output<number>;
    /**
     * The IP address source for pools in the public scope. Only used for provisioning IP address CIDRs to pools in the public scope. Valid values are `byoip` or `amazon`. Default is `byoip`.
     */
    readonly publicIpSource: pulumi.Output<string | undefined>;
    /**
     * Defines whether or not IPv6 pool space is publicly advertisable over the internet. This argument is required if `addressFamily = "ipv6"` and `publicIpSource = "byoip"`, default is `false`. This option is not available for IPv4 pool space or if `publicIpSource = "amazon"`. Setting this argument to `true` when it is not available may result in erroneous differences being reported.
     */
    readonly publiclyAdvertisable: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the source IPAM pool. Use this argument to create a child pool within an existing pool.
     */
    readonly sourceIpamPoolId: pulumi.Output<string | undefined>;
    /**
     * The ID of the IPAM
     */
    readonly state: pulumi.Output<string>;
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
     * Create a VpcIpamPool resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcIpamPoolArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcIpamPool resources.
 */
export interface VpcIpamPoolState {
    /**
     * The IP protocol assigned to this pool. You must choose either IPv4 or IPv6 protocol for a pool.
     */
    addressFamily?: pulumi.Input<string>;
    /**
     * A default netmask length for allocations added to this pool. If, for example, the CIDR assigned to this pool is 10.0.0.0/8 and you enter 16 here, new allocations will default to 10.0.0.0/16 (unless you provide a different netmask value when you create the new allocation).
     */
    allocationDefaultNetmaskLength?: pulumi.Input<number>;
    /**
     * The maximum netmask length that will be required for CIDR allocations in this pool.
     */
    allocationMaxNetmaskLength?: pulumi.Input<number>;
    /**
     * The minimum netmask length that will be required for CIDR allocations in this pool.
     */
    allocationMinNetmaskLength?: pulumi.Input<number>;
    /**
     * Tags that are required for resources that use CIDRs from this IPAM pool. Resources that do not have these tags will not be allowed to allocate space from the pool. If the resources have their tags changed after they have allocated space or if the allocation tagging requirements are changed on the pool, the resource may be marked as noncompliant.
     */
    allocationResourceTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Amazon Resource Name (ARN) of IPAM
     */
    arn?: pulumi.Input<string>;
    /**
     * If you include this argument, IPAM automatically imports any VPCs you have in your scope that fall
     * within the CIDR range in the pool.
     */
    autoImport?: pulumi.Input<boolean>;
    /**
     * Limits which AWS service the pool can be used in. Only useable on public scopes. Valid Values: `ec2`.
     */
    awsService?: pulumi.Input<string>;
    /**
     * Enables you to quickly delete an IPAM pool and all resources within that pool, including provisioned CIDRs, allocations, and other pools.
     */
    cascade?: pulumi.Input<boolean>;
    /**
     * A description for the IPAM pool.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the scope in which you would like to create the IPAM pool.
     */
    ipamScopeId?: pulumi.Input<string>;
    ipamScopeType?: pulumi.Input<string>;
    /**
     * The locale in which you would like to create the IPAM pool. Locale is the Region where you want to make an IPAM pool available for allocations. You can only create pools with locales that match the operating Regions of the IPAM. You can only create VPCs from a pool whose locale matches the VPC's Region. Possible values: Any AWS region, such as `us-east-1`.
     */
    locale?: pulumi.Input<string>;
    poolDepth?: pulumi.Input<number>;
    /**
     * The IP address source for pools in the public scope. Only used for provisioning IP address CIDRs to pools in the public scope. Valid values are `byoip` or `amazon`. Default is `byoip`.
     */
    publicIpSource?: pulumi.Input<string>;
    /**
     * Defines whether or not IPv6 pool space is publicly advertisable over the internet. This argument is required if `addressFamily = "ipv6"` and `publicIpSource = "byoip"`, default is `false`. This option is not available for IPv4 pool space or if `publicIpSource = "amazon"`. Setting this argument to `true` when it is not available may result in erroneous differences being reported.
     */
    publiclyAdvertisable?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the source IPAM pool. Use this argument to create a child pool within an existing pool.
     */
    sourceIpamPoolId?: pulumi.Input<string>;
    /**
     * The ID of the IPAM
     */
    state?: pulumi.Input<string>;
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
 * The set of arguments for constructing a VpcIpamPool resource.
 */
export interface VpcIpamPoolArgs {
    /**
     * The IP protocol assigned to this pool. You must choose either IPv4 or IPv6 protocol for a pool.
     */
    addressFamily: pulumi.Input<string>;
    /**
     * A default netmask length for allocations added to this pool. If, for example, the CIDR assigned to this pool is 10.0.0.0/8 and you enter 16 here, new allocations will default to 10.0.0.0/16 (unless you provide a different netmask value when you create the new allocation).
     */
    allocationDefaultNetmaskLength?: pulumi.Input<number>;
    /**
     * The maximum netmask length that will be required for CIDR allocations in this pool.
     */
    allocationMaxNetmaskLength?: pulumi.Input<number>;
    /**
     * The minimum netmask length that will be required for CIDR allocations in this pool.
     */
    allocationMinNetmaskLength?: pulumi.Input<number>;
    /**
     * Tags that are required for resources that use CIDRs from this IPAM pool. Resources that do not have these tags will not be allowed to allocate space from the pool. If the resources have their tags changed after they have allocated space or if the allocation tagging requirements are changed on the pool, the resource may be marked as noncompliant.
     */
    allocationResourceTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * If you include this argument, IPAM automatically imports any VPCs you have in your scope that fall
     * within the CIDR range in the pool.
     */
    autoImport?: pulumi.Input<boolean>;
    /**
     * Limits which AWS service the pool can be used in. Only useable on public scopes. Valid Values: `ec2`.
     */
    awsService?: pulumi.Input<string>;
    /**
     * Enables you to quickly delete an IPAM pool and all resources within that pool, including provisioned CIDRs, allocations, and other pools.
     */
    cascade?: pulumi.Input<boolean>;
    /**
     * A description for the IPAM pool.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the scope in which you would like to create the IPAM pool.
     */
    ipamScopeId: pulumi.Input<string>;
    /**
     * The locale in which you would like to create the IPAM pool. Locale is the Region where you want to make an IPAM pool available for allocations. You can only create pools with locales that match the operating Regions of the IPAM. You can only create VPCs from a pool whose locale matches the VPC's Region. Possible values: Any AWS region, such as `us-east-1`.
     */
    locale?: pulumi.Input<string>;
    /**
     * The IP address source for pools in the public scope. Only used for provisioning IP address CIDRs to pools in the public scope. Valid values are `byoip` or `amazon`. Default is `byoip`.
     */
    publicIpSource?: pulumi.Input<string>;
    /**
     * Defines whether or not IPv6 pool space is publicly advertisable over the internet. This argument is required if `addressFamily = "ipv6"` and `publicIpSource = "byoip"`, default is `false`. This option is not available for IPv4 pool space or if `publicIpSource = "amazon"`. Setting this argument to `true` when it is not available may result in erroneous differences being reported.
     */
    publiclyAdvertisable?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the source IPAM pool. Use this argument to create a child pool within an existing pool.
     */
    sourceIpamPoolId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
