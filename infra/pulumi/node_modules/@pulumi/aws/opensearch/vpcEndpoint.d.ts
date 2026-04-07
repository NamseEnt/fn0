import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an [AWS Opensearch VPC Endpoint](https://docs.aws.amazon.com/opensearch-service/latest/APIReference/API_CreateVpcEndpoint.html). Creates an Amazon OpenSearch Service-managed VPC endpoint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.opensearch.VpcEndpoint("foo", {
 *     domainArn: domain1.arn,
 *     vpcOptions: {
 *         securityGroupIds: [
 *             test.id,
 *             test2.id,
 *         ],
 *         subnetIds: [
 *             testAwsSubnet.id,
 *             test2AwsSubnet.id,
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import OpenSearch VPC endpoint connections using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/vpcEndpoint:VpcEndpoint example endpoint-id
 * ```
 */
export declare class VpcEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing VpcEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcEndpointState, opts?: pulumi.CustomResourceOptions): VpcEndpoint;
    /**
     * Returns true if the given object is an instance of VpcEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcEndpoint;
    /**
     * Specifies the Amazon Resource Name (ARN) of the domain to create the endpoint for
     */
    readonly domainArn: pulumi.Output<string>;
    /**
     * The connection endpoint ID for connecting to the domain.
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Options to specify the subnets and security groups for the endpoint.
     */
    readonly vpcOptions: pulumi.Output<outputs.opensearch.VpcEndpointVpcOptions>;
    /**
     * Create a VpcEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcEndpoint resources.
 */
export interface VpcEndpointState {
    /**
     * Specifies the Amazon Resource Name (ARN) of the domain to create the endpoint for
     */
    domainArn?: pulumi.Input<string>;
    /**
     * The connection endpoint ID for connecting to the domain.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Options to specify the subnets and security groups for the endpoint.
     */
    vpcOptions?: pulumi.Input<inputs.opensearch.VpcEndpointVpcOptions>;
}
/**
 * The set of arguments for constructing a VpcEndpoint resource.
 */
export interface VpcEndpointArgs {
    /**
     * Specifies the Amazon Resource Name (ARN) of the domain to create the endpoint for
     */
    domainArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Options to specify the subnets and security groups for the endpoint.
     */
    vpcOptions: pulumi.Input<inputs.opensearch.VpcEndpointVpcOptions>;
}
