import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager Connection.
 *
 * Use this resource to create a connection between two devices in your global network.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.Connection("example", {
 *     globalNetworkId: exampleAwsNetworkmanagerGlobalNetwork.id,
 *     deviceId: example1.id,
 *     connectedDeviceId: example2.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_connection` using the connection ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/connection:Connection example arn:aws:networkmanager::123456789012:device/global-network-0d47f6t230mz46dy4/connection-07f6fd08867abc123
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
     * ARN of the connection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the second device in the connection.
     */
    readonly connectedDeviceId: pulumi.Output<string>;
    /**
     * ID of the link for the second device.
     */
    readonly connectedLinkId: pulumi.Output<string | undefined>;
    /**
     * Description of the connection.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ID of the first device in the connection.
     */
    readonly deviceId: pulumi.Output<string>;
    /**
     * ID of the global network.
     *
     * The following arguments are optional:
     */
    readonly globalNetworkId: pulumi.Output<string>;
    /**
     * ID of the link for the first device.
     */
    readonly linkId: pulumi.Output<string | undefined>;
    /**
     * Key-value tags for the connection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
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
     * ARN of the connection.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the second device in the connection.
     */
    connectedDeviceId?: pulumi.Input<string>;
    /**
     * ID of the link for the second device.
     */
    connectedLinkId?: pulumi.Input<string>;
    /**
     * Description of the connection.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the first device in the connection.
     */
    deviceId?: pulumi.Input<string>;
    /**
     * ID of the global network.
     *
     * The following arguments are optional:
     */
    globalNetworkId?: pulumi.Input<string>;
    /**
     * ID of the link for the first device.
     */
    linkId?: pulumi.Input<string>;
    /**
     * Key-value tags for the connection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Connection resource.
 */
export interface ConnectionArgs {
    /**
     * ID of the second device in the connection.
     */
    connectedDeviceId: pulumi.Input<string>;
    /**
     * ID of the link for the second device.
     */
    connectedLinkId?: pulumi.Input<string>;
    /**
     * Description of the connection.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the first device in the connection.
     */
    deviceId: pulumi.Input<string>;
    /**
     * ID of the global network.
     *
     * The following arguments are optional:
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * ID of the link for the first device.
     */
    linkId?: pulumi.Input<string>;
    /**
     * Key-value tags for the connection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
