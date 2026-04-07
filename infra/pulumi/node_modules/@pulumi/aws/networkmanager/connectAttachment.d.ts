import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Network Manager Connect Attachment.
 *
 * Use this resource to create a Connect attachment in AWS Network Manager. Connect attachments enable you to connect your on-premises networks to your core network through a VPC or Transit Gateway attachment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.VpcAttachment("example", {
 *     subnetArns: exampleAwsSubnet.map(__item => __item.arn),
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 * });
 * const exampleConnectAttachment = new aws.networkmanager.ConnectAttachment("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     transportAttachmentId: example.id,
 *     edgeLocation: example.edgeLocation,
 *     options: {
 *         protocol: "GRE",
 *     },
 * });
 * ```
 *
 * ### Usage with attachment accepter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.VpcAttachment("example", {
 *     subnetArns: exampleAwsSubnet.map(__item => __item.arn),
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 * });
 * const exampleAttachmentAccepter = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: example.id,
 *     attachmentType: example.attachmentType,
 * });
 * const exampleConnectAttachment = new aws.networkmanager.ConnectAttachment("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     transportAttachmentId: example.id,
 *     edgeLocation: example.edgeLocation,
 *     options: {
 *         protocol: "GRE",
 *     },
 * }, {
 *     dependsOn: [exampleAttachmentAccepter],
 * });
 * const example2 = new aws.networkmanager.AttachmentAccepter("example2", {
 *     attachmentId: exampleConnectAttachment.id,
 *     attachmentType: exampleConnectAttachment.attachmentType,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_connect_attachment` using the attachment ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/connectAttachment:ConnectAttachment example attachment-0f8fa60d2238d1bd8
 * ```
 */
export declare class ConnectAttachment extends pulumi.CustomResource {
    /**
     * Get an existing ConnectAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectAttachmentState, opts?: pulumi.CustomResourceOptions): ConnectAttachment;
    /**
     * Returns true if the given object is an instance of ConnectAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConnectAttachment;
    /**
     * ARN of the attachment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the attachment.
     */
    readonly attachmentId: pulumi.Output<string>;
    /**
     * Policy rule number associated with the attachment.
     */
    readonly attachmentPolicyRuleNumber: pulumi.Output<number>;
    /**
     * Type of attachment.
     */
    readonly attachmentType: pulumi.Output<string>;
    /**
     * ARN of a core network.
     */
    readonly coreNetworkArn: pulumi.Output<string>;
    /**
     * ID of a core network where you want to create the attachment.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * Region where the edge is located.
     */
    readonly edgeLocation: pulumi.Output<string>;
    /**
     * Options block. See options for more information.
     */
    readonly options: pulumi.Output<outputs.networkmanager.ConnectAttachmentOptions>;
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
     * ID of the attachment between the two connections.
     *
     * The following arguments are optional:
     */
    readonly transportAttachmentId: pulumi.Output<string>;
    /**
     * Create a ConnectAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConnectAttachment resources.
 */
export interface ConnectAttachmentState {
    /**
     * ARN of the attachment.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the attachment.
     */
    attachmentId?: pulumi.Input<string>;
    /**
     * Policy rule number associated with the attachment.
     */
    attachmentPolicyRuleNumber?: pulumi.Input<number>;
    /**
     * Type of attachment.
     */
    attachmentType?: pulumi.Input<string>;
    /**
     * ARN of a core network.
     */
    coreNetworkArn?: pulumi.Input<string>;
    /**
     * ID of a core network where you want to create the attachment.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * Region where the edge is located.
     */
    edgeLocation?: pulumi.Input<string>;
    /**
     * Options block. See options for more information.
     */
    options?: pulumi.Input<inputs.networkmanager.ConnectAttachmentOptions>;
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
     * ID of the attachment between the two connections.
     *
     * The following arguments are optional:
     */
    transportAttachmentId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConnectAttachment resource.
 */
export interface ConnectAttachmentArgs {
    /**
     * ID of a core network where you want to create the attachment.
     */
    coreNetworkId: pulumi.Input<string>;
    /**
     * Region where the edge is located.
     */
    edgeLocation: pulumi.Input<string>;
    /**
     * Options block. See options for more information.
     */
    options: pulumi.Input<inputs.networkmanager.ConnectAttachmentOptions>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ID of the attachment between the two connections.
     *
     * The following arguments are optional:
     */
    transportAttachmentId: pulumi.Input<string>;
}
