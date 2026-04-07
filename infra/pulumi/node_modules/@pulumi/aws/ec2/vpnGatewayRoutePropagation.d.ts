import * as pulumi from "@pulumi/pulumi";
/**
 * Requests automatic route propagation between a VPN gateway and a route table.
 *
 * > **Note:** This resource should not be used with a route table that has
 * the `propagatingVgws` argument set. If that argument is set, any route
 * propagation not explicitly listed in its value will be removed.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.VpnGatewayRoutePropagation("example", {
 *     vpnGatewayId: exampleAwsVpnGateway.id,
 *     routeTableId: exampleAwsRouteTable.id,
 * });
 * ```
 */
export declare class VpnGatewayRoutePropagation extends pulumi.CustomResource {
    /**
     * Get an existing VpnGatewayRoutePropagation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpnGatewayRoutePropagationState, opts?: pulumi.CustomResourceOptions): VpnGatewayRoutePropagation;
    /**
     * Returns true if the given object is an instance of VpnGatewayRoutePropagation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpnGatewayRoutePropagation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The id of the `aws.ec2.RouteTable` to propagate routes into.
     */
    readonly routeTableId: pulumi.Output<string>;
    /**
     * The id of the `aws.ec2.VpnGateway` to propagate routes from.
     */
    readonly vpnGatewayId: pulumi.Output<string>;
    /**
     * Create a VpnGatewayRoutePropagation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpnGatewayRoutePropagationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpnGatewayRoutePropagation resources.
 */
export interface VpnGatewayRoutePropagationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The id of the `aws.ec2.RouteTable` to propagate routes into.
     */
    routeTableId?: pulumi.Input<string>;
    /**
     * The id of the `aws.ec2.VpnGateway` to propagate routes from.
     */
    vpnGatewayId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpnGatewayRoutePropagation resource.
 */
export interface VpnGatewayRoutePropagationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The id of the `aws.ec2.RouteTable` to propagate routes into.
     */
    routeTableId: pulumi.Input<string>;
    /**
     * The id of the `aws.ec2.VpnGateway` to propagate routes from.
     */
    vpnGatewayId: pulumi.Input<string>;
}
