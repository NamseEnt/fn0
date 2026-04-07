import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Network Manager Attachment Accepter.
 *
 * Use this resource to accept cross-account attachments in AWS Network Manager. When an attachment is created in one account and needs to be accepted by another account that owns the core network, this resource handles the acceptance process.
 *
 * ## Example Usage
 *
 * ### VPC Attachment
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: exampleAwsNetworkmanagerVpcAttachment.id,
 *     attachmentType: exampleAwsNetworkmanagerVpcAttachment.attachmentType,
 * });
 * ```
 *
 * ### Site-to-Site VPN Attachment
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: exampleAwsNetworkmanagerSiteToSiteVpnAttachment.id,
 *     attachmentType: exampleAwsNetworkmanagerSiteToSiteVpnAttachment.attachmentType,
 * });
 * ```
 *
 * ### Connect Attachment
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: exampleAwsNetworkmanagerConnectAttachment.id,
 *     attachmentType: exampleAwsNetworkmanagerConnectAttachment.attachmentType,
 * });
 * ```
 *
 * ### Transit Gateway Route Table Attachment
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: exampleAwsNetworkmanagerTransitGatewayRouteTableAttachment.id,
 *     attachmentType: exampleAwsNetworkmanagerTransitGatewayRouteTableAttachment.attachmentType,
 * });
 * ```
 *
 * ### Direct Connect Gateway Attachment
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: exampleAwsNetworkmanagerDxGatewayAttachment.id,
 *     attachmentType: exampleAwsNetworkmanagerDxGatewayAttachment.attachmentType,
 * });
 * ```
 */
export declare class AttachmentAccepter extends pulumi.CustomResource {
    /**
     * Get an existing AttachmentAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AttachmentAccepterState, opts?: pulumi.CustomResourceOptions): AttachmentAccepter;
    /**
     * Returns true if the given object is an instance of AttachmentAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AttachmentAccepter;
    /**
     * ID of the attachment.
     */
    readonly attachmentId: pulumi.Output<string>;
    /**
     * Policy rule number associated with the attachment.
     */
    readonly attachmentPolicyRuleNumber: pulumi.Output<number>;
    /**
     * Type of attachment. Valid values: `CONNECT`, `DIRECT_CONNECT_GATEWAY`, `SITE_TO_SITE_VPN`, `TRANSIT_GATEWAY_ROUTE_TABLE`, `VPC`.
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
     * Region where the edge is located. This is returned for all attachment types except Direct Connect gateway attachments, which instead return `edgeLocations`.
     */
    readonly edgeLocation: pulumi.Output<string>;
    /**
     * Edge locations that the Direct Connect gateway is associated with. This is returned only for Direct Connect gateway attachments. All other attachment types return `edgeLocation`.
     */
    readonly edgeLocations: pulumi.Output<string[]>;
    /**
     * ID of the attachment account owner.
     */
    readonly ownerAccountId: pulumi.Output<string>;
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
     * Create a AttachmentAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AttachmentAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AttachmentAccepter resources.
 */
export interface AttachmentAccepterState {
    /**
     * ID of the attachment.
     */
    attachmentId?: pulumi.Input<string>;
    /**
     * Policy rule number associated with the attachment.
     */
    attachmentPolicyRuleNumber?: pulumi.Input<number>;
    /**
     * Type of attachment. Valid values: `CONNECT`, `DIRECT_CONNECT_GATEWAY`, `SITE_TO_SITE_VPN`, `TRANSIT_GATEWAY_ROUTE_TABLE`, `VPC`.
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
     * Region where the edge is located. This is returned for all attachment types except Direct Connect gateway attachments, which instead return `edgeLocations`.
     */
    edgeLocation?: pulumi.Input<string>;
    /**
     * Edge locations that the Direct Connect gateway is associated with. This is returned only for Direct Connect gateway attachments. All other attachment types return `edgeLocation`.
     */
    edgeLocations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ID of the attachment account owner.
     */
    ownerAccountId?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a AttachmentAccepter resource.
 */
export interface AttachmentAccepterArgs {
    /**
     * ID of the attachment.
     */
    attachmentId: pulumi.Input<string>;
    /**
     * Type of attachment. Valid values: `CONNECT`, `DIRECT_CONNECT_GATEWAY`, `SITE_TO_SITE_VPN`, `TRANSIT_GATEWAY_ROUTE_TABLE`, `VPC`.
     */
    attachmentType: pulumi.Input<string>;
}
