import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the accepter's side of an EC2 Transit Gateway Peering Attachment.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.PeeringAttachmentAccepter("example", {
 *     transitGatewayAttachmentId: exampleAwsEc2TransitGatewayPeeringAttachment.id,
 *     tags: {
 *         Name: "Example cross-account attachment",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_peering_attachment_accepter` using the EC2 Transit Gateway Attachment identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/peeringAttachmentAccepter:PeeringAttachmentAccepter example tgw-attach-12345678
 * ```
 */
export declare class PeeringAttachmentAccepter extends pulumi.CustomResource {
    /**
     * Get an existing PeeringAttachmentAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PeeringAttachmentAccepterState, opts?: pulumi.CustomResourceOptions): PeeringAttachmentAccepter;
    /**
     * Returns true if the given object is an instance of PeeringAttachmentAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PeeringAttachmentAccepter;
    /**
     * Identifier of the AWS account that owns the EC2 TGW peering.
     */
    readonly peerAccountId: pulumi.Output<string>;
    readonly peerRegion: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway to peer with.
     */
    readonly peerTransitGatewayId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * The ID of the EC2 Transit Gateway Peering Attachment to manage.
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway.
     */
    readonly transitGatewayId: pulumi.Output<string>;
    /**
     * Create a PeeringAttachmentAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PeeringAttachmentAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PeeringAttachmentAccepter resources.
 */
export interface PeeringAttachmentAccepterState {
    /**
     * Identifier of the AWS account that owns the EC2 TGW peering.
     */
    peerAccountId?: pulumi.Input<string>;
    peerRegion?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway to peer with.
     */
    peerTransitGatewayId?: pulumi.Input<string>;
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
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the EC2 Transit Gateway Peering Attachment to manage.
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway.
     */
    transitGatewayId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PeeringAttachmentAccepter resource.
 */
export interface PeeringAttachmentAccepterArgs {
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
     * The ID of the EC2 Transit Gateway Peering Attachment to manage.
     */
    transitGatewayAttachmentId: pulumi.Input<string>;
}
