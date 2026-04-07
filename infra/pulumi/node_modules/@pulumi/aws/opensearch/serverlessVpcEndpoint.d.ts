import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS OpenSearchServerless VPC Endpoint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.opensearch.ServerlessVpcEndpoint("example", {
 *     name: "myendpoint",
 *     subnetIds: [exampleAwsSubnet.id],
 *     vpcId: exampleAwsVpc.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import OpenSearchServerless Vpc Endpointa using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/serverlessVpcEndpoint:ServerlessVpcEndpoint example vpce-8012925589
 * ```
 */
export declare class ServerlessVpcEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing ServerlessVpcEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServerlessVpcEndpointState, opts?: pulumi.CustomResourceOptions): ServerlessVpcEndpoint;
    /**
     * Returns true if the given object is an instance of ServerlessVpcEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServerlessVpcEndpoint;
    /**
     * Name of the interface endpoint.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * One or more security groups that define the ports, protocols, and sources for inbound traffic that you are authorizing into your endpoint. Up to 5 security groups can be provided.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * One or more subnet IDs from which you'll access OpenSearch Serverless. Up to 6 subnets can be provided.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    readonly timeouts: pulumi.Output<outputs.opensearch.ServerlessVpcEndpointTimeouts | undefined>;
    /**
     * ID of the VPC from which you'll access OpenSearch Serverless.
     *
     * The following arguments are optional:
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a ServerlessVpcEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServerlessVpcEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServerlessVpcEndpoint resources.
 */
export interface ServerlessVpcEndpointState {
    /**
     * Name of the interface endpoint.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more security groups that define the ports, protocols, and sources for inbound traffic that you are authorizing into your endpoint. Up to 5 security groups can be provided.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more subnet IDs from which you'll access OpenSearch Serverless. Up to 6 subnets can be provided.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    timeouts?: pulumi.Input<inputs.opensearch.ServerlessVpcEndpointTimeouts>;
    /**
     * ID of the VPC from which you'll access OpenSearch Serverless.
     *
     * The following arguments are optional:
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServerlessVpcEndpoint resource.
 */
export interface ServerlessVpcEndpointArgs {
    /**
     * Name of the interface endpoint.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more security groups that define the ports, protocols, and sources for inbound traffic that you are authorizing into your endpoint. Up to 5 security groups can be provided.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more subnet IDs from which you'll access OpenSearch Serverless. Up to 6 subnets can be provided.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    timeouts?: pulumi.Input<inputs.opensearch.ServerlessVpcEndpointTimeouts>;
    /**
     * ID of the VPC from which you'll access OpenSearch Serverless.
     *
     * The following arguments are optional:
     */
    vpcId: pulumi.Input<string>;
}
