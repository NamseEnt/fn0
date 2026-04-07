import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to accept a pending VPC Endpoint Connection accept request to VPC Endpoint Service.
 *
 * ## Example Usage
 *
 * ### Accept cross-account request
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.VpcEndpointService("example", {
 *     acceptanceRequired: false,
 *     networkLoadBalancerArns: [exampleAwsLb.arn],
 * });
 * const exampleVpcEndpoint = new aws.ec2.VpcEndpoint("example", {
 *     vpcId: testAlternate.id,
 *     serviceName: testAwsVpcEndpointService.serviceName,
 *     vpcEndpointType: "Interface",
 *     privateDnsEnabled: false,
 *     securityGroupIds: [test.id],
 * });
 * const exampleVpcEndpointConnectionAccepter = new aws.ec2.VpcEndpointConnectionAccepter("example", {
 *     vpcEndpointServiceId: example.id,
 *     vpcEndpointId: exampleVpcEndpoint.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Endpoint Services using ID of the connection, which is the `VPC Endpoint Service ID` and `VPC Endpoint ID` separated by underscore (`_`).. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcEndpointConnectionAccepter:VpcEndpointConnectionAccepter foo vpce-svc-0f97a19d3fa8220bc_vpce-010601a6db371e263
 * ```
 */
export declare class VpcEndpointConnectionAccepter extends pulumi.CustomResource {
    /**
     * Get an existing VpcEndpointConnectionAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcEndpointConnectionAccepterState, opts?: pulumi.CustomResourceOptions): VpcEndpointConnectionAccepter;
    /**
     * Returns true if the given object is an instance of VpcEndpointConnectionAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcEndpointConnectionAccepter;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * AWS VPC Endpoint ID.
     */
    readonly vpcEndpointId: pulumi.Output<string>;
    /**
     * AWS VPC Endpoint Service ID.
     */
    readonly vpcEndpointServiceId: pulumi.Output<string>;
    /**
     * State of the VPC Endpoint.
     */
    readonly vpcEndpointState: pulumi.Output<string>;
    /**
     * Create a VpcEndpointConnectionAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcEndpointConnectionAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcEndpointConnectionAccepter resources.
 */
export interface VpcEndpointConnectionAccepterState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS VPC Endpoint ID.
     */
    vpcEndpointId?: pulumi.Input<string>;
    /**
     * AWS VPC Endpoint Service ID.
     */
    vpcEndpointServiceId?: pulumi.Input<string>;
    /**
     * State of the VPC Endpoint.
     */
    vpcEndpointState?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcEndpointConnectionAccepter resource.
 */
export interface VpcEndpointConnectionAccepterArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS VPC Endpoint ID.
     */
    vpcEndpointId: pulumi.Input<string>;
    /**
     * AWS VPC Endpoint Service ID.
     */
    vpcEndpointServiceId: pulumi.Input<string>;
}
