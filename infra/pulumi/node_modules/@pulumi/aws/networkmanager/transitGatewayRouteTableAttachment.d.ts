import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager transit gateway route table attachment.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.TransitGatewayRouteTableAttachment("example", {
 *     peeringId: exampleAwsNetworkmanagerTransitGatewayPeering.id,
 *     transitGatewayRouteTableArn: exampleAwsEc2TransitGatewayRouteTable.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_transit_gateway_route_table_attachment` using the attachment ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/transitGatewayRouteTableAttachment:TransitGatewayRouteTableAttachment example attachment-0f8fa60d2238d1bd8
 * ```
 */
export declare class TransitGatewayRouteTableAttachment extends pulumi.CustomResource {
    /**
     * Get an existing TransitGatewayRouteTableAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TransitGatewayRouteTableAttachmentState, opts?: pulumi.CustomResourceOptions): TransitGatewayRouteTableAttachment;
    /**
     * Returns true if the given object is an instance of TransitGatewayRouteTableAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TransitGatewayRouteTableAttachment;
    /**
     * Attachment ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Policy rule number associated with the attachment.
     */
    readonly attachmentPolicyRuleNumber: pulumi.Output<number>;
    /**
     * Type of attachment.
     */
    readonly attachmentType: pulumi.Output<string>;
    /**
     * ARN of the core network.
     */
    readonly coreNetworkArn: pulumi.Output<string>;
    /**
     * ID of the core network.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * Edge location for the peer.
     */
    readonly edgeLocation: pulumi.Output<string>;
    /**
     * ID of the attachment account owner.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * ID of the peer for the attachment.
     */
    readonly peeringId: pulumi.Output<string>;
    /**
     * Attachment resource ARN.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Name of the segment attachment.
     */
    readonly segmentName: pulumi.Output<string>;
    /**
     * State of the attachment.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ARN of the transit gateway route table for the attachment.
     *
     * The following arguments are optional:
     */
    readonly transitGatewayRouteTableArn: pulumi.Output<string>;
    /**
     * Create a TransitGatewayRouteTableAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TransitGatewayRouteTableAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TransitGatewayRouteTableAttachment resources.
 */
export interface TransitGatewayRouteTableAttachmentState {
    /**
     * Attachment ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Policy rule number associated with the attachment.
     */
    attachmentPolicyRuleNumber?: pulumi.Input<number>;
    /**
     * Type of attachment.
     */
    attachmentType?: pulumi.Input<string>;
    /**
     * ARN of the core network.
     */
    coreNetworkArn?: pulumi.Input<string>;
    /**
     * ID of the core network.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * Edge location for the peer.
     */
    edgeLocation?: pulumi.Input<string>;
    /**
     * ID of the attachment account owner.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * ID of the peer for the attachment.
     */
    peeringId?: pulumi.Input<string>;
    /**
     * Attachment resource ARN.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Name of the segment attachment.
     */
    segmentName?: pulumi.Input<string>;
    /**
     * State of the attachment.
     */
    state?: pulumi.Input<string>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ARN of the transit gateway route table for the attachment.
     *
     * The following arguments are optional:
     */
    transitGatewayRouteTableArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TransitGatewayRouteTableAttachment resource.
 */
export interface TransitGatewayRouteTableAttachmentArgs {
    /**
     * ID of the peer for the attachment.
     */
    peeringId: pulumi.Input<string>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of the transit gateway route table for the attachment.
     *
     * The following arguments are optional:
     */
    transitGatewayRouteTableArn: pulumi.Input<string>;
}
