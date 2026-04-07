import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway Route.
 *
 * ## Example Usage
 *
 * ### Standard usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.Route("example", {
 *     destinationCidrBlock: "0.0.0.0/0",
 *     transitGatewayAttachmentId: exampleAwsEc2TransitGatewayVpcAttachment.id,
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGateway.associationDefaultRouteTableId,
 * });
 * ```
 *
 * ### Blackhole route
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.Route("example", {
 *     destinationCidrBlock: "0.0.0.0/0",
 *     blackhole: true,
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGateway.associationDefaultRouteTableId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_route` using the EC2 Transit Gateway Route Table, an underscore, and the destination. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/route:Route example tgw-rtb-12345678_0.0.0.0/0
 * ```
 */
export declare class Route extends pulumi.CustomResource {
    /**
     * Get an existing Route resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteState, opts?: pulumi.CustomResourceOptions): Route;
    /**
     * Returns true if the given object is an instance of Route.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Route;
    /**
     * Indicates whether to drop traffic that matches this route (default to `false`).
     */
    readonly blackhole: pulumi.Output<boolean | undefined>;
    /**
     * IPv4 or IPv6 RFC1924 CIDR used for destination matches. Routing decisions are based on the most specific match.
     */
    readonly destinationCidrBlock: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment (required if `blackhole` is set to false).
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string | undefined>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     */
    readonly transitGatewayRouteTableId: pulumi.Output<string>;
    /**
     * Create a Route resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Route resources.
 */
export interface RouteState {
    /**
     * Indicates whether to drop traffic that matches this route (default to `false`).
     */
    blackhole?: pulumi.Input<boolean>;
    /**
     * IPv4 or IPv6 RFC1924 CIDR used for destination matches. Routing decisions are based on the most specific match.
     */
    destinationCidrBlock?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment (required if `blackhole` is set to false).
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     */
    transitGatewayRouteTableId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Route resource.
 */
export interface RouteArgs {
    /**
     * Indicates whether to drop traffic that matches this route (default to `false`).
     */
    blackhole?: pulumi.Input<boolean>;
    /**
     * IPv4 or IPv6 RFC1924 CIDR used for destination matches. Routing decisions are based on the most specific match.
     */
    destinationCidrBlock: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Attachment (required if `blackhole` is set to false).
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     */
    transitGatewayRouteTableId: pulumi.Input<string>;
}
