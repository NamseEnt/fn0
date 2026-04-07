import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an EC2 Transit Gateway Peering Attachment.
 * For examples of custom route table association and propagation, see the [EC2 Transit Gateway Networking Examples Guide](https://docs.aws.amazon.com/vpc/latest/tgw/TGW_Scenarios.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const peer = aws.getRegion({});
 * const local = new aws.ec2transitgateway.TransitGateway("local", {tags: {
 *     Name: "Local TGW",
 * }});
 * const peerTransitGateway = new aws.ec2transitgateway.TransitGateway("peer", {tags: {
 *     Name: "Peer TGW",
 * }});
 * const example = new aws.ec2transitgateway.PeeringAttachment("example", {
 *     peerAccountId: peerTransitGateway.ownerId,
 *     peerRegion: peer.then(peer => peer.name),
 *     peerTransitGatewayId: peerTransitGateway.id,
 *     transitGatewayId: local.id,
 *     tags: {
 *         Name: "TGW Peering Requestor",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_peering_attachment` using the EC2 Transit Gateway Attachment identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/peeringAttachment:PeeringAttachment example tgw-attach-12345678
 * ```
 */
export declare class PeeringAttachment extends pulumi.CustomResource {
    /**
     * Get an existing PeeringAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PeeringAttachmentState, opts?: pulumi.CustomResourceOptions): PeeringAttachment;
    /**
     * Returns true if the given object is an instance of PeeringAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PeeringAttachment;
    /**
     * ARN of the attachment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Describes whether dynamic routing is enabled or disabled for the transit gateway peering request. See options below for more details!
     */
    readonly options: pulumi.Output<outputs.ec2transitgateway.PeeringAttachmentOptions | undefined>;
    /**
     * Account ID of EC2 Transit Gateway to peer with. Defaults to the account ID the AWS provider is currently connected to.
     */
    readonly peerAccountId: pulumi.Output<string>;
    /**
     * Region of EC2 Transit Gateway to peer with.
     */
    readonly peerRegion: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway to peer with.
     */
    readonly peerTransitGatewayId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly state: pulumi.Output<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Peering Attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Identifier of EC2 Transit Gateway.
     */
    readonly transitGatewayId: pulumi.Output<string>;
    /**
     * Create a PeeringAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PeeringAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PeeringAttachment resources.
 */
export interface PeeringAttachmentState {
    /**
     * ARN of the attachment.
     */
    arn?: pulumi.Input<string>;
    /**
     * Describes whether dynamic routing is enabled or disabled for the transit gateway peering request. See options below for more details!
     */
    options?: pulumi.Input<inputs.ec2transitgateway.PeeringAttachmentOptions>;
    /**
     * Account ID of EC2 Transit Gateway to peer with. Defaults to the account ID the AWS provider is currently connected to.
     */
    peerAccountId?: pulumi.Input<string>;
    /**
     * Region of EC2 Transit Gateway to peer with.
     */
    peerRegion?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway to peer with.
     */
    peerTransitGatewayId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    state?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Peering Attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Identifier of EC2 Transit Gateway.
     */
    transitGatewayId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PeeringAttachment resource.
 */
export interface PeeringAttachmentArgs {
    /**
     * Describes whether dynamic routing is enabled or disabled for the transit gateway peering request. See options below for more details!
     */
    options?: pulumi.Input<inputs.ec2transitgateway.PeeringAttachmentOptions>;
    /**
     * Account ID of EC2 Transit Gateway to peer with. Defaults to the account ID the AWS provider is currently connected to.
     */
    peerAccountId?: pulumi.Input<string>;
    /**
     * Region of EC2 Transit Gateway to peer with.
     */
    peerRegion: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway to peer with.
     */
    peerTransitGatewayId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Peering Attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Identifier of EC2 Transit Gateway.
     */
    transitGatewayId: pulumi.Input<string>;
}
