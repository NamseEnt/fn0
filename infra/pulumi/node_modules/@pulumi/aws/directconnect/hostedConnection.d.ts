import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a hosted connection on the specified interconnect or a link aggregation group (LAG) of interconnects. Intended for use by AWS Direct Connect Partners only.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const hosted = new aws.directconnect.HostedConnection("hosted", {
 *     connectionId: "dxcon-ffabc123",
 *     bandwidth: "100Mbps",
 *     name: "tf-dx-hosted-connection",
 *     ownerAccountId: "123456789012",
 *     vlan: 1,
 * });
 * ```
 */
export declare class HostedConnection extends pulumi.CustomResource {
    /**
     * Get an existing HostedConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostedConnectionState, opts?: pulumi.CustomResourceOptions): HostedConnection;
    /**
     * Returns true if the given object is an instance of HostedConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HostedConnection;
    /**
     * The Direct Connect endpoint on which the physical connection terminates.
     */
    readonly awsDevice: pulumi.Output<string>;
    /**
     * The bandwidth of the connection. Valid values for dedicated connections: 1Gbps, 10Gbps. Valid values for hosted connections: 50Mbps, 100Mbps, 200Mbps, 300Mbps, 400Mbps, 500Mbps, 1Gbps, 2Gbps, 5Gbps, 10Gbps, and 25Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Hosted Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/hosted_connection.html).
     */
    readonly bandwidth: pulumi.Output<string>;
    /**
     * The ID of the interconnect or LAG.
     */
    readonly connectionId: pulumi.Output<string>;
    /**
     * The AWS Region where the connection is located.
     */
    readonly connectionRegion: pulumi.Output<string>;
    /**
     * Indicates whether the connection supports a secondary BGP peer in the same address family (IPv4/IPv6).
     */
    readonly hasLogicalRedundancy: pulumi.Output<string>;
    /**
     * Boolean value representing if jumbo frames have been enabled for this connection.
     */
    readonly jumboFrameCapable: pulumi.Output<boolean>;
    /**
     * The ID of the LAG.
     */
    readonly lagId: pulumi.Output<string>;
    /**
     * The time of the most recent call to [DescribeLoa](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLoa.html) for this connection.
     */
    readonly loaIssueTime: pulumi.Output<string>;
    /**
     * The location of the connection.
     */
    readonly location: pulumi.Output<string>;
    /**
     * The name of the connection.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ID of the AWS account of the customer for the connection.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * The name of the AWS Direct Connect service provider associated with the connection.
     */
    readonly partnerName: pulumi.Output<string>;
    /**
     * The name of the service provider associated with the connection.
     */
    readonly providerName: pulumi.Output<string>;
    /**
     * (**Deprecated**) The AWS Region where the connection is located. Use `connectionRegion` instead.
     *
     * @deprecated region is deprecated. Use connectionRegion instead.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The state of the connection. Possible values include: ordering, requested, pending, available, down, deleting, deleted, rejected, unknown. See [AllocateHostedConnection](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_AllocateHostedConnection.html) for a description of each connection state.
     */
    readonly state: pulumi.Output<string>;
    /**
     * The dedicated VLAN provisioned to the hosted connection.
     */
    readonly vlan: pulumi.Output<number>;
    /**
     * Create a HostedConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostedConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HostedConnection resources.
 */
export interface HostedConnectionState {
    /**
     * The Direct Connect endpoint on which the physical connection terminates.
     */
    awsDevice?: pulumi.Input<string>;
    /**
     * The bandwidth of the connection. Valid values for dedicated connections: 1Gbps, 10Gbps. Valid values for hosted connections: 50Mbps, 100Mbps, 200Mbps, 300Mbps, 400Mbps, 500Mbps, 1Gbps, 2Gbps, 5Gbps, 10Gbps, and 25Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Hosted Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/hosted_connection.html).
     */
    bandwidth?: pulumi.Input<string>;
    /**
     * The ID of the interconnect or LAG.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * The AWS Region where the connection is located.
     */
    connectionRegion?: pulumi.Input<string>;
    /**
     * Indicates whether the connection supports a secondary BGP peer in the same address family (IPv4/IPv6).
     */
    hasLogicalRedundancy?: pulumi.Input<string>;
    /**
     * Boolean value representing if jumbo frames have been enabled for this connection.
     */
    jumboFrameCapable?: pulumi.Input<boolean>;
    /**
     * The ID of the LAG.
     */
    lagId?: pulumi.Input<string>;
    /**
     * The time of the most recent call to [DescribeLoa](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLoa.html) for this connection.
     */
    loaIssueTime?: pulumi.Input<string>;
    /**
     * The location of the connection.
     */
    location?: pulumi.Input<string>;
    /**
     * The name of the connection.
     */
    name?: pulumi.Input<string>;
    /**
     * The ID of the AWS account of the customer for the connection.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * The name of the AWS Direct Connect service provider associated with the connection.
     */
    partnerName?: pulumi.Input<string>;
    /**
     * The name of the service provider associated with the connection.
     */
    providerName?: pulumi.Input<string>;
    /**
     * (**Deprecated**) The AWS Region where the connection is located. Use `connectionRegion` instead.
     *
     * @deprecated region is deprecated. Use connectionRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * The state of the connection. Possible values include: ordering, requested, pending, available, down, deleting, deleted, rejected, unknown. See [AllocateHostedConnection](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_AllocateHostedConnection.html) for a description of each connection state.
     */
    state?: pulumi.Input<string>;
    /**
     * The dedicated VLAN provisioned to the hosted connection.
     */
    vlan?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a HostedConnection resource.
 */
export interface HostedConnectionArgs {
    /**
     * The bandwidth of the connection. Valid values for dedicated connections: 1Gbps, 10Gbps. Valid values for hosted connections: 50Mbps, 100Mbps, 200Mbps, 300Mbps, 400Mbps, 500Mbps, 1Gbps, 2Gbps, 5Gbps, 10Gbps, and 25Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Hosted Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/hosted_connection.html).
     */
    bandwidth: pulumi.Input<string>;
    /**
     * The ID of the interconnect or LAG.
     */
    connectionId: pulumi.Input<string>;
    /**
     * The name of the connection.
     */
    name?: pulumi.Input<string>;
    /**
     * The ID of the AWS account of the customer for the connection.
     */
    ownerAccountId: pulumi.Input<string>;
    /**
     * The dedicated VLAN provisioned to the hosted connection.
     */
    vlan: pulumi.Input<number>;
}
