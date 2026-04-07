import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway Route Table propagation.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.RouteTablePropagation("example", {
 *     transitGatewayAttachmentId: exampleAwsEc2TransitGatewayVpcAttachment.id,
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGatewayRouteTable.id,
 * });
 * ```
 *
 * ### Direct Connect Gateway Propagation
 *
 * When propagating routes from a Direct Connect Gateway attachment, reference the `transitGatewayAttachmentId` attribute directly from the `aws.directconnect.GatewayAssociation` resource (available in v6.5.0+):
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Gateway("example", {
 *     name: "example",
 *     amazonSideAsn: "64512",
 * });
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {description: "example"});
 * const exampleGatewayAssociation = new aws.directconnect.GatewayAssociation("example", {
 *     dxGatewayId: example.id,
 *     associatedGatewayId: exampleTransitGateway.id,
 *     allowedPrefixes: ["10.0.0.0/16"],
 * });
 * const exampleRouteTable = new aws.ec2transitgateway.RouteTable("example", {transitGatewayId: exampleTransitGateway.id});
 * // Correct: Reference the attachment ID directly from the association resource
 * const exampleRouteTablePropagation = new aws.ec2transitgateway.RouteTablePropagation("example", {
 *     transitGatewayAttachmentId: exampleGatewayAssociation.transitGatewayAttachmentId,
 *     transitGatewayRouteTableId: exampleRouteTable.id,
 * });
 * ```
 *
 * > **NOTE:** Avoid using the `aws.ec2transitgateway.getDirectConnectGatewayAttachment` data source to retrieve the attachment ID, as this can cause unnecessary resource recreation when unrelated attributes of the Direct Connect Gateway association change (such as `allowedPrefixes`). Always reference the `transitGatewayAttachmentId` attribute directly from the `aws.directconnect.GatewayAssociation` resource when available.
 *
 * ### VPC Attachment Propagation
 *
 * For VPC attachments, always reference the attachment resource's `id` attribute directly. Avoid using data sources or lifecycle rules that might cause the attachment ID to become unknown during planning:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Vpc("example", {cidrBlock: "10.0.0.0/16"});
 * const exampleSubnet = new aws.ec2.Subnet("example", {
 *     vpcId: example.id,
 *     cidrBlock: "10.0.1.0/24",
 * });
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {description: "example"});
 * const exampleVpcAttachment = new aws.ec2transitgateway.VpcAttachment("example", {
 *     subnetIds: [exampleSubnet.id],
 *     transitGatewayId: exampleTransitGateway.id,
 *     vpcId: example.id,
 * });
 * const exampleRouteTable = new aws.ec2transitgateway.RouteTable("example", {transitGatewayId: exampleTransitGateway.id});
 * // Correct: Reference the VPC attachment ID directly
 * const exampleRouteTablePropagation = new aws.ec2transitgateway.RouteTablePropagation("example", {
 *     transitGatewayAttachmentId: exampleVpcAttachment.id,
 *     transitGatewayRouteTableId: exampleRouteTable.id,
 * });
 * ```
 *
 * > **NOTE:** When the `transitGatewayAttachmentId` changes (for example, when a VPC attachment is replaced), this resource will be recreated. This is the correct behavior to maintain consistency between the attachment and its route table propagation.
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_route_table_propagation` using the EC2 Transit Gateway Route Table identifier, an underscore, and the EC2 Transit Gateway Attachment identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/routeTablePropagation:RouteTablePropagation example tgw-rtb-12345678_tgw-attach-87654321
 * ```
 */
export declare class RouteTablePropagation extends pulumi.CustomResource {
    /**
     * Get an existing RouteTablePropagation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteTablePropagationState, opts?: pulumi.CustomResourceOptions): RouteTablePropagation;
    /**
     * Returns true if the given object is an instance of RouteTablePropagation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RouteTablePropagation;
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
     * Identifier of EC2 Transit Gateway Route Table.
     */
    readonly transitGatewayRouteTableId: pulumi.Output<string>;
    /**
     * Create a RouteTablePropagation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteTablePropagationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RouteTablePropagation resources.
 */
export interface RouteTablePropagationState {
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
     * Identifier of EC2 Transit Gateway Route Table.
     */
    transitGatewayRouteTableId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RouteTablePropagation resource.
 */
export interface RouteTablePropagationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment.
     */
    transitGatewayAttachmentId: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     */
    transitGatewayRouteTableId: pulumi.Input<string>;
}
