import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway Connect.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.VpcAttachment("example", {
 *     subnetIds: [exampleAwsSubnet.id],
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 *     vpcId: exampleAwsVpc.id,
 * });
 * const attachment = new aws.ec2transitgateway.Connect("attachment", {
 *     transportAttachmentId: example.id,
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_connect` using the EC2 Transit Gateway Connect identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/connect:Connect example tgw-attach-12345678
 * ```
 */
export declare class Connect extends pulumi.CustomResource {
    /**
     * Get an existing Connect resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectState, opts?: pulumi.CustomResourceOptions): Connect;
    /**
     * Returns true if the given object is an instance of Connect.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Connect;
    /**
     * The tunnel protocol. Valid values: `gre`. Default is `gre`.
     */
    readonly protocol: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Connect. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Boolean whether the Connect should be associated with the EC2 Transit Gateway association default route table. This cannot be configured or perform drift detection with Resource Access Manager shared EC2 Transit Gateways. Default value: `true`.
     */
    readonly transitGatewayDefaultRouteTableAssociation: pulumi.Output<boolean | undefined>;
    /**
     * Boolean whether the Connect should propagate routes with the EC2 Transit Gateway propagation default route table. This cannot be configured or perform drift detection with Resource Access Manager shared EC2 Transit Gateways. Default value: `true`.
     */
    readonly transitGatewayDefaultRouteTablePropagation: pulumi.Output<boolean | undefined>;
    /**
     * Identifier of EC2 Transit Gateway.
     */
    readonly transitGatewayId: pulumi.Output<string>;
    /**
     * The underlaying VPC attachment
     */
    readonly transportAttachmentId: pulumi.Output<string>;
    /**
     * Create a Connect resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Connect resources.
 */
export interface ConnectState {
    /**
     * The tunnel protocol. Valid values: `gre`. Default is `gre`.
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Connect. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Boolean whether the Connect should be associated with the EC2 Transit Gateway association default route table. This cannot be configured or perform drift detection with Resource Access Manager shared EC2 Transit Gateways. Default value: `true`.
     */
    transitGatewayDefaultRouteTableAssociation?: pulumi.Input<boolean>;
    /**
     * Boolean whether the Connect should propagate routes with the EC2 Transit Gateway propagation default route table. This cannot be configured or perform drift detection with Resource Access Manager shared EC2 Transit Gateways. Default value: `true`.
     */
    transitGatewayDefaultRouteTablePropagation?: pulumi.Input<boolean>;
    /**
     * Identifier of EC2 Transit Gateway.
     */
    transitGatewayId?: pulumi.Input<string>;
    /**
     * The underlaying VPC attachment
     */
    transportAttachmentId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Connect resource.
 */
export interface ConnectArgs {
    /**
     * The tunnel protocol. Valid values: `gre`. Default is `gre`.
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Connect. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Boolean whether the Connect should be associated with the EC2 Transit Gateway association default route table. This cannot be configured or perform drift detection with Resource Access Manager shared EC2 Transit Gateways. Default value: `true`.
     */
    transitGatewayDefaultRouteTableAssociation?: pulumi.Input<boolean>;
    /**
     * Boolean whether the Connect should propagate routes with the EC2 Transit Gateway propagation default route table. This cannot be configured or perform drift detection with Resource Access Manager shared EC2 Transit Gateways. Default value: `true`.
     */
    transitGatewayDefaultRouteTablePropagation?: pulumi.Input<boolean>;
    /**
     * Identifier of EC2 Transit Gateway.
     */
    transitGatewayId: pulumi.Input<string>;
    /**
     * The underlaying VPC attachment
     */
    transportAttachmentId: pulumi.Input<string>;
}
