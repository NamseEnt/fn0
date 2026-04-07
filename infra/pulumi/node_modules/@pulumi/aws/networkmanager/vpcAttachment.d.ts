import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Network Manager VPC attachment.
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
 *     subnetArns: [exampleAwsSubnet.arn],
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 * });
 * ```
 *
 * ### Usage with Options
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.VpcAttachment("example", {
 *     subnetArns: [exampleAwsSubnet.arn],
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 *     options: {
 *         applianceModeSupport: false,
 *         dnsSupport: true,
 *         ipv6Support: false,
 *         securityGroupReferencingSupport: true,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_vpc_attachment` using the attachment ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/vpcAttachment:VpcAttachment example attachment-0f8fa60d2238d1bd8
 * ```
 */
export declare class VpcAttachment extends pulumi.CustomResource {
    /**
     * Get an existing VpcAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcAttachmentState, opts?: pulumi.CustomResourceOptions): VpcAttachment;
    /**
     * Returns true if the given object is an instance of VpcAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcAttachment;
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
     * ARN of a core network.
     */
    readonly coreNetworkArn: pulumi.Output<string>;
    /**
     * ID of a core network for the VPC attachment.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * Region where the edge is located.
     */
    readonly edgeLocation: pulumi.Output<string>;
    /**
     * Options for the VPC attachment. See below.
     */
    readonly options: pulumi.Output<outputs.networkmanager.VpcAttachmentOptions>;
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
     * Subnet ARNs of the VPC attachment.
     */
    readonly subnetArns: pulumi.Output<string[]>;
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
     * ARN of the VPC.
     *
     * The following arguments are optional:
     */
    readonly vpcArn: pulumi.Output<string>;
    /**
     * Create a VpcAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcAttachment resources.
 */
export interface VpcAttachmentState {
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
     * ARN of a core network.
     */
    coreNetworkArn?: pulumi.Input<string>;
    /**
     * ID of a core network for the VPC attachment.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * Region where the edge is located.
     */
    edgeLocation?: pulumi.Input<string>;
    /**
     * Options for the VPC attachment. See below.
     */
    options?: pulumi.Input<inputs.networkmanager.VpcAttachmentOptions>;
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
     * Subnet ARNs of the VPC attachment.
     */
    subnetArns?: pulumi.Input<pulumi.Input<string>[]>;
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
     * ARN of the VPC.
     *
     * The following arguments are optional:
     */
    vpcArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcAttachment resource.
 */
export interface VpcAttachmentArgs {
    /**
     * ID of a core network for the VPC attachment.
     */
    coreNetworkId: pulumi.Input<string>;
    /**
     * Options for the VPC attachment. See below.
     */
    options?: pulumi.Input<inputs.networkmanager.VpcAttachmentOptions>;
    /**
     * Subnet ARNs of the VPC attachment.
     */
    subnetArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of the VPC.
     *
     * The following arguments are optional:
     */
    vpcArn: pulumi.Input<string>;
}
