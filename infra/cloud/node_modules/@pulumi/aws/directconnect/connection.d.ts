import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Connection of Direct Connect.
 *
 * ## Example Usage
 *
 * ### Create a connection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const hoge = new aws.directconnect.Connection("hoge", {
 *     name: "tf-dx-connection",
 *     bandwidth: "1Gbps",
 *     location: "EqDC2",
 * });
 * ```
 *
 * ### Request a MACsec-capable connection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Connection("example", {
 *     name: "tf-dx-connection",
 *     bandwidth: "10Gbps",
 *     location: "EqDA2",
 *     requestMacsec: true,
 * });
 * ```
 *
 * ### Configure encryption mode for MACsec-capable connections
 *
 * > **NOTE:** You can only specify the `encryptionMode` argument once the connection is in an `Available` state.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Connection("example", {
 *     name: "tf-dx-connection",
 *     bandwidth: "10Gbps",
 *     location: "EqDC2",
 *     requestMacsec: true,
 *     encryptionMode: "must_encrypt",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Direct Connect connections using the connection `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:directconnect/connection:Connection test_connection dxcon-ffre0ec3
 * ```
 */
export declare class Connection extends pulumi.CustomResource {
    /**
     * Get an existing Connection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectionState, opts?: pulumi.CustomResourceOptions): Connection;
    /**
     * Returns true if the given object is an instance of Connection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Connection;
    /**
     * The ARN of the connection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Direct Connect endpoint on which the physical connection terminates.
     */
    readonly awsDevice: pulumi.Output<string>;
    /**
     * The bandwidth of the connection. Valid values for dedicated connections: 1Gbps, 10Gbps, 100Gbps, and 400Gbps. Valid values for hosted connections: 50Mbps, 100Mbps, 200Mbps, 300Mbps, 400Mbps, 500Mbps, 1Gbps, 2Gbps, 5Gbps, 10Gbps, and 25Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Dedicated Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/dedicated_connection.html) and [Hosted Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/hosted_connection.html).
     */
    readonly bandwidth: pulumi.Output<string>;
    /**
     * The connection MAC Security (MACsec) encryption mode. MAC Security (MACsec) is only available on dedicated connections. Valid values are `noEncrypt`, `shouldEncrypt`, and `mustEncrypt`.
     */
    readonly encryptionMode: pulumi.Output<string>;
    /**
     * Indicates whether the connection supports a secondary BGP peer in the same address family (IPv4/IPv6).
     */
    readonly hasLogicalRedundancy: pulumi.Output<string>;
    /**
     * Boolean value representing if jumbo frames have been enabled for this connection.
     */
    readonly jumboFrameCapable: pulumi.Output<boolean>;
    /**
     * The AWS Direct Connect location where the connection is located. See [DescribeLocations](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLocations.html) for the list of AWS Direct Connect locations. Use `locationCode`.
     */
    readonly location: pulumi.Output<string>;
    /**
     * Boolean value indicating whether the connection supports MAC Security (MACsec).
     */
    readonly macsecCapable: pulumi.Output<boolean>;
    /**
     * The name of the connection.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the connection.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * The name of the AWS Direct Connect service provider associated with the connection.
     */
    readonly partnerName: pulumi.Output<string>;
    /**
     * The MAC Security (MACsec) port link status of the connection.
     */
    readonly portEncryptionStatus: pulumi.Output<string>;
    /**
     * The name of the service provider associated with the connection.
     */
    readonly providerName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Boolean value indicating whether you want the connection to support MAC Security (MACsec). MAC Security (MACsec) is only available on dedicated connections. See [MACsec prerequisites](https://docs.aws.amazon.com/directconnect/latest/UserGuide/direct-connect-mac-sec-getting-started.html#mac-sec-prerequisites) for more information about MAC Security (MACsec) prerequisites. Default value: `false`.
     *
     * > **NOTE:** Changing the value of `requestMacsec` will cause the resource to be destroyed and re-created.
     */
    readonly requestMacsec: pulumi.Output<boolean | undefined>;
    /**
     * Set to true if you do not wish the connection to be deleted at destroy time, and instead just removed from the state.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
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
     * The VLAN ID.
     */
    readonly vlanId: pulumi.Output<number>;
    /**
     * Create a Connection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Connection resources.
 */
export interface ConnectionState {
    /**
     * The ARN of the connection.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Direct Connect endpoint on which the physical connection terminates.
     */
    awsDevice?: pulumi.Input<string>;
    /**
     * The bandwidth of the connection. Valid values for dedicated connections: 1Gbps, 10Gbps, 100Gbps, and 400Gbps. Valid values for hosted connections: 50Mbps, 100Mbps, 200Mbps, 300Mbps, 400Mbps, 500Mbps, 1Gbps, 2Gbps, 5Gbps, 10Gbps, and 25Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Dedicated Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/dedicated_connection.html) and [Hosted Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/hosted_connection.html).
     */
    bandwidth?: pulumi.Input<string>;
    /**
     * The connection MAC Security (MACsec) encryption mode. MAC Security (MACsec) is only available on dedicated connections. Valid values are `noEncrypt`, `shouldEncrypt`, and `mustEncrypt`.
     */
    encryptionMode?: pulumi.Input<string>;
    /**
     * Indicates whether the connection supports a secondary BGP peer in the same address family (IPv4/IPv6).
     */
    hasLogicalRedundancy?: pulumi.Input<string>;
    /**
     * Boolean value representing if jumbo frames have been enabled for this connection.
     */
    jumboFrameCapable?: pulumi.Input<boolean>;
    /**
     * The AWS Direct Connect location where the connection is located. See [DescribeLocations](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLocations.html) for the list of AWS Direct Connect locations. Use `locationCode`.
     */
    location?: pulumi.Input<string>;
    /**
     * Boolean value indicating whether the connection supports MAC Security (MACsec).
     */
    macsecCapable?: pulumi.Input<boolean>;
    /**
     * The name of the connection.
     */
    name?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the connection.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * The name of the AWS Direct Connect service provider associated with the connection.
     */
    partnerName?: pulumi.Input<string>;
    /**
     * The MAC Security (MACsec) port link status of the connection.
     */
    portEncryptionStatus?: pulumi.Input<string>;
    /**
     * The name of the service provider associated with the connection.
     */
    providerName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Boolean value indicating whether you want the connection to support MAC Security (MACsec). MAC Security (MACsec) is only available on dedicated connections. See [MACsec prerequisites](https://docs.aws.amazon.com/directconnect/latest/UserGuide/direct-connect-mac-sec-getting-started.html#mac-sec-prerequisites) for more information about MAC Security (MACsec) prerequisites. Default value: `false`.
     *
     * > **NOTE:** Changing the value of `requestMacsec` will cause the resource to be destroyed and re-created.
     */
    requestMacsec?: pulumi.Input<boolean>;
    /**
     * Set to true if you do not wish the connection to be deleted at destroy time, and instead just removed from the state.
     */
    skipDestroy?: pulumi.Input<boolean>;
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
    /**
     * The VLAN ID.
     */
    vlanId?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Connection resource.
 */
export interface ConnectionArgs {
    /**
     * The bandwidth of the connection. Valid values for dedicated connections: 1Gbps, 10Gbps, 100Gbps, and 400Gbps. Valid values for hosted connections: 50Mbps, 100Mbps, 200Mbps, 300Mbps, 400Mbps, 500Mbps, 1Gbps, 2Gbps, 5Gbps, 10Gbps, and 25Gbps. Case sensitive. Refer to the AWS Direct Connection supported bandwidths for [Dedicated Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/dedicated_connection.html) and [Hosted Connections](https://docs.aws.amazon.com/directconnect/latest/UserGuide/hosted_connection.html).
     */
    bandwidth: pulumi.Input<string>;
    /**
     * The connection MAC Security (MACsec) encryption mode. MAC Security (MACsec) is only available on dedicated connections. Valid values are `noEncrypt`, `shouldEncrypt`, and `mustEncrypt`.
     */
    encryptionMode?: pulumi.Input<string>;
    /**
     * The AWS Direct Connect location where the connection is located. See [DescribeLocations](https://docs.aws.amazon.com/directconnect/latest/APIReference/API_DescribeLocations.html) for the list of AWS Direct Connect locations. Use `locationCode`.
     */
    location: pulumi.Input<string>;
    /**
     * The name of the connection.
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the service provider associated with the connection.
     */
    providerName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Boolean value indicating whether you want the connection to support MAC Security (MACsec). MAC Security (MACsec) is only available on dedicated connections. See [MACsec prerequisites](https://docs.aws.amazon.com/directconnect/latest/UserGuide/direct-connect-mac-sec-getting-started.html#mac-sec-prerequisites) for more information about MAC Security (MACsec) prerequisites. Default value: `false`.
     *
     * > **NOTE:** Changing the value of `requestMacsec` will cause the resource to be destroyed and re-created.
     */
    requestMacsec?: pulumi.Input<boolean>;
    /**
     * Set to true if you do not wish the connection to be deleted at destroy time, and instead just removed from the state.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
