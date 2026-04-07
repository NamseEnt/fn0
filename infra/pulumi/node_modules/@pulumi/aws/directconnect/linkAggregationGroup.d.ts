import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Direct Connect LAG. Connections can be added to the LAG via the `aws.directconnect.Connection` and `aws.directconnect.ConnectionAssociation` resources.
 *
 * > *NOTE:* When creating a LAG, if no existing connection is specified, Direct Connect will create a connection and this provider will remove this unmanaged connection during resource creation.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const hoge = new aws.directconnect.LinkAggregationGroup("hoge", {
 *     name: "tf-dx-lag",
 *     connectionsBandwidth: "1Gbps",
 *     location: "EqDC2",
 *     forceDestroy: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Direct Connect LAGs using the LAG `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:directconnect/linkAggregationGroup:LinkAggregationGroup test_lag dxlag-fgnsp5rq
 * ```
 */
export declare class LinkAggregationGroup extends pulumi.CustomResource {
    /**
     * Get an existing LinkAggregationGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LinkAggregationGroupState, opts?: pulumi.CustomResourceOptions): LinkAggregationGroup;
    /**
     * Returns true if the given object is an instance of LinkAggregationGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LinkAggregationGroup;
    /**
     * The ARN of the LAG.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ID of an existing dedicated connection to migrate to the LAG.
     */
    readonly connectionId: pulumi.Output<string | undefined>;
    /**
     * The bandwidth of the individual dedicated connections bundled by the LAG. Valid values: 1Gbps, 10Gbps, 100Gbps, and 400Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Dedicated Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/dedicated_connection.html).
     */
    readonly connectionsBandwidth: pulumi.Output<string>;
    /**
     * A boolean that indicates all connections associated with the LAG should be deleted so that the LAG can be destroyed without error. These objects are *not* recoverable.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Indicates whether the LAG supports a secondary BGP peer in the same address family (IPv4/IPv6).
     */
    readonly hasLogicalRedundancy: pulumi.Output<string>;
    /**
     * Indicates whether jumbo frames (9001 MTU) are supported.
     */
    readonly jumboFrameCapable: pulumi.Output<boolean>;
    /**
     * The AWS Direct Connect location in which the LAG should be allocated. See [DescribeLocations](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLocations.html) for the list of AWS Direct Connect locations. Use `locationCode`.
     */
    readonly location: pulumi.Output<string>;
    /**
     * The name of the LAG.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the LAG.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * The name of the service provider associated with the LAG.
     */
    readonly providerName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Create a LinkAggregationGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LinkAggregationGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LinkAggregationGroup resources.
 */
export interface LinkAggregationGroupState {
    /**
     * The ARN of the LAG.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ID of an existing dedicated connection to migrate to the LAG.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * The bandwidth of the individual dedicated connections bundled by the LAG. Valid values: 1Gbps, 10Gbps, 100Gbps, and 400Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Dedicated Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/dedicated_connection.html).
     */
    connectionsBandwidth?: pulumi.Input<string>;
    /**
     * A boolean that indicates all connections associated with the LAG should be deleted so that the LAG can be destroyed without error. These objects are *not* recoverable.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Indicates whether the LAG supports a secondary BGP peer in the same address family (IPv4/IPv6).
     */
    hasLogicalRedundancy?: pulumi.Input<string>;
    /**
     * Indicates whether jumbo frames (9001 MTU) are supported.
     */
    jumboFrameCapable?: pulumi.Input<boolean>;
    /**
     * The AWS Direct Connect location in which the LAG should be allocated. See [DescribeLocations](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLocations.html) for the list of AWS Direct Connect locations. Use `locationCode`.
     */
    location?: pulumi.Input<string>;
    /**
     * The name of the LAG.
     */
    name?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the LAG.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * The name of the service provider associated with the LAG.
     */
    providerName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
 * The set of arguments for constructing a LinkAggregationGroup resource.
 */
export interface LinkAggregationGroupArgs {
    /**
     * The ID of an existing dedicated connection to migrate to the LAG.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * The bandwidth of the individual dedicated connections bundled by the LAG. Valid values: 1Gbps, 10Gbps, 100Gbps, and 400Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Dedicated Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/dedicated_connection.html).
     */
    connectionsBandwidth: pulumi.Input<string>;
    /**
     * A boolean that indicates all connections associated with the LAG should be deleted so that the LAG can be destroyed without error. These objects are *not* recoverable.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The AWS Direct Connect location in which the LAG should be allocated. See [DescribeLocations](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLocations.html) for the list of AWS Direct Connect locations. Use `locationCode`.
     */
    location: pulumi.Input<string>;
    /**
     * The name of the LAG.
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the service provider associated with the LAG.
     */
    providerName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
