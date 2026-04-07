import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway Policy Table association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.PolicyTableAssociation("example", {
 *     transitGatewayAttachmentId: exampleAwsNetworkmanagerTransitGatewayPeering.transitGatewayPeeringAttachmentId,
 *     transitGatewayPolicyTableId: exampleAwsEc2TransitGatewayPolicyTable.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_policy_table_association` using the EC2 Transit Gateway Policy Table identifier, an underscore, and the EC2 Transit Gateway Attachment identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/policyTableAssociation:PolicyTableAssociation example tgw-rtb-12345678_tgw-attach-87654321
 * ```
 */
export declare class PolicyTableAssociation extends pulumi.CustomResource {
    /**
     * Get an existing PolicyTableAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyTableAssociationState, opts?: pulumi.CustomResourceOptions): PolicyTableAssociation;
    /**
     * Returns true if the given object is an instance of PolicyTableAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PolicyTableAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the resource
     */
    readonly resourceId: pulumi.Output<string>;
    /**
     * Type of the resource
     */
    readonly resourceType: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway Policy Table.
     */
    readonly transitGatewayPolicyTableId: pulumi.Output<string>;
    /**
     * Create a PolicyTableAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyTableAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PolicyTableAssociation resources.
 */
export interface PolicyTableAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the resource
     */
    resourceId?: pulumi.Input<string>;
    /**
     * Type of the resource
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Policy Table.
     */
    transitGatewayPolicyTableId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PolicyTableAssociation resource.
 */
export interface PolicyTableAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    transitGatewayAttachmentId: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Policy Table.
     */
    transitGatewayPolicyTableId: pulumi.Input<string>;
}
