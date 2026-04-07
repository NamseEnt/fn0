import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager transit gateway peering connection. Creates a peering connection between an AWS Cloud WAN core network and an AWS Transit Gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.TransitGatewayPeering("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     transitGatewayArn: exampleAwsEc2TransitGateway.arn,
 * }, {
 *     dependsOn: [
 *         exampleAwsEc2TransitGatewayPolicyTable,
 *         exampleAwsNetworkmanagerCoreNetworkPolicyAttachment,
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_transit_gateway_peering` using the peering ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/transitGatewayPeering:TransitGatewayPeering example peering-444555aaabbb11223
 * ```
 */
export declare class TransitGatewayPeering extends pulumi.CustomResource {
    /**
     * Get an existing TransitGatewayPeering resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TransitGatewayPeeringState, opts?: pulumi.CustomResourceOptions): TransitGatewayPeering;
    /**
     * Returns true if the given object is an instance of TransitGatewayPeering.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TransitGatewayPeering;
    /**
     * Peering ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of the core network.
     */
    readonly coreNetworkArn: pulumi.Output<string>;
    /**
     * ID of a core network.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * Edge location for the peer.
     */
    readonly edgeLocation: pulumi.Output<string>;
    /**
     * ID of the account owner.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * Type of peering. This will be `TRANSIT_GATEWAY`.
     */
    readonly peeringType: pulumi.Output<string>;
    /**
     * Resource ARN of the peer.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Key-value tags for the peering. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ARN of the transit gateway for the peering request.
     *
     * The following arguments are optional:
     */
    readonly transitGatewayArn: pulumi.Output<string>;
    /**
     * ID of the transit gateway peering attachment.
     */
    readonly transitGatewayPeeringAttachmentId: pulumi.Output<string>;
    /**
     * Create a TransitGatewayPeering resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TransitGatewayPeeringArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TransitGatewayPeering resources.
 */
export interface TransitGatewayPeeringState {
    /**
     * Peering ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the core network.
     */
    coreNetworkArn?: pulumi.Input<string>;
    /**
     * ID of a core network.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * Edge location for the peer.
     */
    edgeLocation?: pulumi.Input<string>;
    /**
     * ID of the account owner.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * Type of peering. This will be `TRANSIT_GATEWAY`.
     */
    peeringType?: pulumi.Input<string>;
    /**
     * Resource ARN of the peer.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Key-value tags for the peering. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * ARN of the transit gateway for the peering request.
     *
     * The following arguments are optional:
     */
    transitGatewayArn?: pulumi.Input<string>;
    /**
     * ID of the transit gateway peering attachment.
     */
    transitGatewayPeeringAttachmentId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TransitGatewayPeering resource.
 */
export interface TransitGatewayPeeringArgs {
    /**
     * ID of a core network.
     */
    coreNetworkId: pulumi.Input<string>;
    /**
     * Key-value tags for the peering. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of the transit gateway for the peering request.
     *
     * The following arguments are optional:
     */
    transitGatewayArn: pulumi.Input<string>;
}
