import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager transit gateway registration. Registers a transit gateway to a global network. The transit gateway can be in any AWS Region, but it must be owned by the same AWS account that owns the global network. You cannot register a transit gateway in more than one global network.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.GlobalNetwork("example", {description: "example"});
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {});
 * const exampleTransitGatewayRegistration = new aws.networkmanager.TransitGatewayRegistration("example", {
 *     globalNetworkId: example.id,
 *     transitGatewayArn: exampleTransitGateway.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_transit_gateway_registration` using the global network ID and transit gateway ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/transitGatewayRegistration:TransitGatewayRegistration example global-network-0d47f6t230mz46dy4,arn:aws:ec2:us-west-2:123456789012:transit-gateway/tgw-123abc05e04123abc
 * ```
 */
export declare class TransitGatewayRegistration extends pulumi.CustomResource {
    /**
     * Get an existing TransitGatewayRegistration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TransitGatewayRegistrationState, opts?: pulumi.CustomResourceOptions): TransitGatewayRegistration;
    /**
     * Returns true if the given object is an instance of TransitGatewayRegistration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TransitGatewayRegistration;
    /**
     * ID of the Global Network to register to.
     */
    readonly globalNetworkId: pulumi.Output<string>;
    /**
     * ARN of the Transit Gateway to register.
     */
    readonly transitGatewayArn: pulumi.Output<string>;
    /**
     * Create a TransitGatewayRegistration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TransitGatewayRegistrationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TransitGatewayRegistration resources.
 */
export interface TransitGatewayRegistrationState {
    /**
     * ID of the Global Network to register to.
     */
    globalNetworkId?: pulumi.Input<string>;
    /**
     * ARN of the Transit Gateway to register.
     */
    transitGatewayArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TransitGatewayRegistration resource.
 */
export interface TransitGatewayRegistrationArgs {
    /**
     * ID of the Global Network to register to.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * ARN of the Transit Gateway to register.
     */
    transitGatewayArn: pulumi.Input<string>;
}
