import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a VPC Endpoint Policy resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getVpcEndpointService({
 *     service: "dynamodb",
 * });
 * const exampleVpc = new aws.ec2.Vpc("example", {cidrBlock: "10.0.0.0/16"});
 * const exampleVpcEndpoint = new aws.ec2.VpcEndpoint("example", {
 *     serviceName: example.then(example => example.serviceName),
 *     vpcId: exampleVpc.id,
 * });
 * const exampleVpcEndpointPolicy = new aws.ec2.VpcEndpointPolicy("example", {
 *     vpcEndpointId: exampleVpcEndpoint.id,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Sid: "AllowAll",
 *             Effect: "Allow",
 *             Principal: {
 *                 AWS: "*",
 *             },
 *             Action: ["dynamodb:*"],
 *             Resource: "*",
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Endpoint Policies using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcEndpointPolicy:VpcEndpointPolicy example vpce-3ecf2a57
 * ```
 */
export declare class VpcEndpointPolicy extends pulumi.CustomResource {
    /**
     * Get an existing VpcEndpointPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcEndpointPolicyState, opts?: pulumi.CustomResourceOptions): VpcEndpointPolicy;
    /**
     * Returns true if the given object is an instance of VpcEndpointPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcEndpointPolicy;
    /**
     * A policy to attach to the endpoint that controls access to the service. Defaults to full access. All `Gateway` and some `Interface` endpoints support policies - see the [relevant AWS documentation](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-endpoints-access.html) for more details.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The VPC Endpoint ID.
     */
    readonly vpcEndpointId: pulumi.Output<string>;
    /**
     * Create a VpcEndpointPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcEndpointPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcEndpointPolicy resources.
 */
export interface VpcEndpointPolicyState {
    /**
     * A policy to attach to the endpoint that controls access to the service. Defaults to full access. All `Gateway` and some `Interface` endpoints support policies - see the [relevant AWS documentation](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-endpoints-access.html) for more details.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The VPC Endpoint ID.
     */
    vpcEndpointId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcEndpointPolicy resource.
 */
export interface VpcEndpointPolicyArgs {
    /**
     * A policy to attach to the endpoint that controls access to the service. Defaults to full access. All `Gateway` and some `Interface` endpoints support policies - see the [relevant AWS documentation](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-endpoints-access.html) for more details.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The VPC Endpoint ID.
     */
    vpcEndpointId: pulumi.Input<string>;
}
