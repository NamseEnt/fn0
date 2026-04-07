import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway Prefix List Reference.
 *
 * ## Example Usage
 *
 * ### Attachment Routing
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.PrefixListReference("example", {
 *     prefixListId: exampleAwsEc2ManagedPrefixList.id,
 *     transitGatewayAttachmentId: exampleAwsEc2TransitGatewayVpcAttachment.id,
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGateway.associationDefaultRouteTableId,
 * });
 * ```
 *
 * ### Blackhole Routing
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.PrefixListReference("example", {
 *     blackhole: true,
 *     prefixListId: exampleAwsEc2ManagedPrefixList.id,
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGateway.associationDefaultRouteTableId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_prefix_list_reference` using the EC2 Transit Gateway Route Table identifier and EC2 Prefix List identifier, separated by an underscore (`_`). For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/prefixListReference:PrefixListReference example tgw-rtb-12345678_pl-12345678
 * ```
 */
export declare class PrefixListReference extends pulumi.CustomResource {
    /**
     * Get an existing PrefixListReference resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PrefixListReferenceState, opts?: pulumi.CustomResourceOptions): PrefixListReference;
    /**
     * Returns true if the given object is an instance of PrefixListReference.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PrefixListReference;
    /**
     * Indicates whether to drop traffic that matches the Prefix List. Defaults to `false`.
     */
    readonly blackhole: pulumi.Output<boolean | undefined>;
    /**
     * Identifier of EC2 Prefix List.
     */
    readonly prefixListId: pulumi.Output<string>;
    readonly prefixListOwnerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string | undefined>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     *
     * The following arguments are optional:
     */
    readonly transitGatewayRouteTableId: pulumi.Output<string>;
    /**
     * Create a PrefixListReference resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PrefixListReferenceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PrefixListReference resources.
 */
export interface PrefixListReferenceState {
    /**
     * Indicates whether to drop traffic that matches the Prefix List. Defaults to `false`.
     */
    blackhole?: pulumi.Input<boolean>;
    /**
     * Identifier of EC2 Prefix List.
     */
    prefixListId?: pulumi.Input<string>;
    prefixListOwnerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     *
     * The following arguments are optional:
     */
    transitGatewayRouteTableId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PrefixListReference resource.
 */
export interface PrefixListReferenceArgs {
    /**
     * Indicates whether to drop traffic that matches the Prefix List. Defaults to `false`.
     */
    blackhole?: pulumi.Input<boolean>;
    /**
     * Identifier of EC2 Prefix List.
     */
    prefixListId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     *
     * The following arguments are optional:
     */
    transitGatewayRouteTableId: pulumi.Input<string>;
}
