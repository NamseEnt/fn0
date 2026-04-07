import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Network Manager Direct Connect Gateway Attachment.
 *
 * Use this resource to create and manage a Direct Connect Gateway attachment to a Cloud WAN core network.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.networkmanager.DxGatewayAttachment("test", {
 *     coreNetworkId: testAwsNetworkmanagerCoreNetworkPolicyAttachment.coreNetworkId,
 *     directConnectGatewayArn: `arn:aws:directconnect::${current.accountId}:dx-gateway/${testAwsDxGateway.id}`,
 *     edgeLocations: [currentAwsRegion.region],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Manager DX Gateway Attachment using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/dxGatewayAttachment:DxGatewayAttachment example attachment-1a2b3c4d5e6f7g
 * ```
 */
export declare class DxGatewayAttachment extends pulumi.CustomResource {
    /**
     * Get an existing DxGatewayAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DxGatewayAttachmentState, opts?: pulumi.CustomResourceOptions): DxGatewayAttachment;
    /**
     * Returns true if the given object is an instance of DxGatewayAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DxGatewayAttachment;
    /**
     * ARN of the attachment.
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
     * ARN of the core network for the attachment.
     */
    readonly coreNetworkArn: pulumi.Output<string>;
    /**
     * ID of the Cloud WAN core network to which the Direct Connect gateway attachment should be attached.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * ARN of the Direct Connect gateway attachment.
     */
    readonly directConnectGatewayArn: pulumi.Output<string>;
    /**
     * One or more core network edge locations to associate with the Direct Connect gateway attachment.
     *
     * The following arguments are optional:
     */
    readonly edgeLocations: pulumi.Output<string[]>;
    /**
     * ID of the attachment account owner.
     */
    readonly ownerAccountId: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.networkmanager.DxGatewayAttachmentTimeouts | undefined>;
    /**
     * Create a DxGatewayAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DxGatewayAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DxGatewayAttachment resources.
 */
export interface DxGatewayAttachmentState {
    /**
     * ARN of the attachment.
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
     * ARN of the core network for the attachment.
     */
    coreNetworkArn?: pulumi.Input<string>;
    /**
     * ID of the Cloud WAN core network to which the Direct Connect gateway attachment should be attached.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * ARN of the Direct Connect gateway attachment.
     */
    directConnectGatewayArn?: pulumi.Input<string>;
    /**
     * One or more core network edge locations to associate with the Direct Connect gateway attachment.
     *
     * The following arguments are optional:
     */
    edgeLocations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ID of the attachment account owner.
     */
    ownerAccountId?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.networkmanager.DxGatewayAttachmentTimeouts>;
}
/**
 * The set of arguments for constructing a DxGatewayAttachment resource.
 */
export interface DxGatewayAttachmentArgs {
    /**
     * ID of the Cloud WAN core network to which the Direct Connect gateway attachment should be attached.
     */
    coreNetworkId: pulumi.Input<string>;
    /**
     * ARN of the Direct Connect gateway attachment.
     */
    directConnectGatewayArn: pulumi.Input<string>;
    /**
     * One or more core network edge locations to associate with the Direct Connect gateway attachment.
     *
     * The following arguments are optional:
     */
    edgeLocations: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.networkmanager.DxGatewayAttachmentTimeouts>;
}
