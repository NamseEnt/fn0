import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager site-to-site VPN attachment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.SiteToSiteVpnAttachment("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpnConnectionArn: exampleAwsVpnConnection.arn,
 * });
 * ```
 *
 * ### Full Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as awscc from "@pulumi/awscc";
 * import * as std from "@pulumi/std";
 *
 * const testCustomerGateway = new aws.ec2.CustomerGateway("test", {
 *     bgpAsn: "65000",
 *     ipAddress: "172.0.0.1",
 *     type: "ipsec.1",
 * });
 * const testVpnConnection = new aws.ec2.VpnConnection("test", {
 *     customerGatewayId: testCustomerGateway.id,
 *     type: "ipsec.1",
 *     tags: {
 *         Name: "test",
 *     },
 * });
 * const testGlobalNetwork = new aws.networkmanager.GlobalNetwork("test", {tags: {
 *     Name: "test",
 * }});
 * const test = aws.networkmanager.getCoreNetworkPolicyDocument({
 *     coreNetworkConfigurations: [{
 *         vpnEcmpSupport: false,
 *         asnRanges: ["64512-64555"],
 *         edgeLocations: [{
 *             location: current.region,
 *             asn: "64512",
 *         }],
 *     }],
 *     segments: [{
 *         name: "shared",
 *         description: "SegmentForSharedServices",
 *         requireAttachmentAcceptance: true,
 *     }],
 *     segmentActions: [{
 *         action: "share",
 *         mode: "attachment-route",
 *         segment: "shared",
 *         shareWiths: ["*"],
 *     }],
 *     attachmentPolicies: [{
 *         ruleNumber: 1,
 *         conditionLogic: "or",
 *         conditions: [{
 *             type: "tag-value",
 *             operator: "equals",
 *             key: "segment",
 *             value: "shared",
 *         }],
 *         action: {
 *             associationMethod: "constant",
 *             segment: "shared",
 *         },
 *     }],
 * });
 * const testNetworkmanagerCoreNetwork = new awscc.index.NetworkmanagerCoreNetwork("test", {
 *     globalNetworkId: testGlobalNetwork.id,
 *     policyDocument: JSON.stringify(std.jsondecode({
 *         input: test.json,
 *     }).result),
 * });
 * const testSiteToSiteVpnAttachment = new aws.networkmanager.SiteToSiteVpnAttachment("test", {
 *     coreNetworkId: testNetworkmanagerCoreNetwork.id,
 *     vpnConnectionArn: testVpnConnection.arn,
 *     tags: {
 *         segment: "shared",
 *     },
 * });
 * const testAttachmentAccepter = new aws.networkmanager.AttachmentAccepter("test", {
 *     attachmentId: testSiteToSiteVpnAttachment.id,
 *     attachmentType: testSiteToSiteVpnAttachment.attachmentType,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_site_to_site_vpn_attachment` using the attachment ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/siteToSiteVpnAttachment:SiteToSiteVpnAttachment example attachment-0f8fa60d2238d1bd8
 * ```
 */
export declare class SiteToSiteVpnAttachment extends pulumi.CustomResource {
    /**
     * Get an existing SiteToSiteVpnAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SiteToSiteVpnAttachmentState, opts?: pulumi.CustomResourceOptions): SiteToSiteVpnAttachment;
    /**
     * Returns true if the given object is an instance of SiteToSiteVpnAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SiteToSiteVpnAttachment;
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
     * ID of a core network for the VPN attachment.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * Region where the edge is located.
     */
    readonly edgeLocation: pulumi.Output<string>;
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
     * ARN of the site-to-site VPN connection.
     *
     * The following arguments are optional:
     */
    readonly vpnConnectionArn: pulumi.Output<string>;
    /**
     * Create a SiteToSiteVpnAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SiteToSiteVpnAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SiteToSiteVpnAttachment resources.
 */
export interface SiteToSiteVpnAttachmentState {
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
     * ID of a core network for the VPN attachment.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * Region where the edge is located.
     */
    edgeLocation?: pulumi.Input<string>;
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
     * ARN of the site-to-site VPN connection.
     *
     * The following arguments are optional:
     */
    vpnConnectionArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SiteToSiteVpnAttachment resource.
 */
export interface SiteToSiteVpnAttachmentArgs {
    /**
     * ID of a core network for the VPN attachment.
     */
    coreNetworkId: pulumi.Input<string>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of the site-to-site VPN connection.
     *
     * The following arguments are optional:
     */
    vpnConnectionArn: pulumi.Input<string>;
}
