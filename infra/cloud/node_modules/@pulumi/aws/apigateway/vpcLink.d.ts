import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an API Gateway VPC Link.
 *
 * > **Note:** Amazon API Gateway Version 1 VPC Links enable private integrations that connect REST APIs to private resources in a VPC.
 * To enable private integration for HTTP APIs, use the Amazon API Gateway Version 2 VPC Link resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lb.LoadBalancer("example", {
 *     name: "example",
 *     internal: true,
 *     loadBalancerType: "network",
 *     subnetMappings: [{
 *         subnetId: "12345",
 *     }],
 * });
 * const exampleVpcLink = new aws.apigateway.VpcLink("example", {
 *     name: "example",
 *     description: "example description",
 *     targetArn: example.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import API Gateway VPC Link using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/vpcLink:VpcLink example 12345abcde
 * ```
 */
export declare class VpcLink extends pulumi.CustomResource {
    /**
     * Get an existing VpcLink resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcLinkState, opts?: pulumi.CustomResourceOptions): VpcLink;
    /**
     * Returns true if the given object is an instance of VpcLink.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcLink;
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the VPC link.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name used to label and identify the VPC link.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * List of network load balancer arns in the VPC targeted by the VPC link. Currently AWS only supports 1 target.
     */
    readonly targetArn: pulumi.Output<string>;
    /**
     * Create a VpcLink resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcLinkArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcLink resources.
 */
export interface VpcLinkState {
    arn?: pulumi.Input<string>;
    /**
     * Description of the VPC link.
     */
    description?: pulumi.Input<string>;
    /**
     * Name used to label and identify the VPC link.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of network load balancer arns in the VPC targeted by the VPC link. Currently AWS only supports 1 target.
     */
    targetArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcLink resource.
 */
export interface VpcLinkArgs {
    /**
     * Description of the VPC link.
     */
    description?: pulumi.Input<string>;
    /**
     * Name used to label and identify the VPC link.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of network load balancer arns in the VPC targeted by the VPC link. Currently AWS only supports 1 target.
     */
    targetArn: pulumi.Input<string>;
}
