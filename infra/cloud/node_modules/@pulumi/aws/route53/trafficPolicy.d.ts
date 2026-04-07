import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Route53 Traffic Policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53.TrafficPolicy("example", {
 *     name: "example",
 *     comment: "example comment",
 *     document: `{
 *   \\"AWSPolicyFormatVersion\\": \\"2015-10-01\\",
 *   \\"RecordType\\": \\"A\\",
 *   \\"Endpoints\\": {
 *     \\"endpoint-start-NkPh\\": {
 *       \\"Type\\": \\"value\\",
 *       \\"Value\\": \\"10.0.0.2\\"
 *     }
 *   },
 *   \\"StartEndpoint\\": \\"endpoint-start-NkPh\\"
 * }
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Traffic Policy using the `id` and `version`. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/trafficPolicy:TrafficPolicy example 01a52019-d16f-422a-ae72-c306d2b6df7e/1
 * ```
 */
export declare class TrafficPolicy extends pulumi.CustomResource {
    /**
     * Get an existing TrafficPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrafficPolicyState, opts?: pulumi.CustomResourceOptions): TrafficPolicy;
    /**
     * Returns true if the given object is an instance of TrafficPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrafficPolicy;
    /**
     * Amazon Resource Name (ARN) of the traffic policy.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Comment for the traffic policy.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * Policy document. This is a JSON formatted string. For more information about building Route53 traffic policy documents, see the [AWS Route53 Traffic Policy document format](https://docs.aws.amazon.com/Route53/latest/APIReference/api-policies-traffic-policy-document-format.html)
     *
     * The following arguments are optional:
     */
    readonly document: pulumi.Output<string>;
    /**
     * Name of the traffic policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * DNS type of the resource record sets that Amazon Route 53 creates when you use a traffic policy to create a traffic policy instance.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Version number of the traffic policy. This value is automatically incremented by AWS after each update of this resource.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a TrafficPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrafficPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrafficPolicy resources.
 */
export interface TrafficPolicyState {
    /**
     * Amazon Resource Name (ARN) of the traffic policy.
     */
    arn?: pulumi.Input<string>;
    /**
     * Comment for the traffic policy.
     */
    comment?: pulumi.Input<string>;
    /**
     * Policy document. This is a JSON formatted string. For more information about building Route53 traffic policy documents, see the [AWS Route53 Traffic Policy document format](https://docs.aws.amazon.com/Route53/latest/APIReference/api-policies-traffic-policy-document-format.html)
     *
     * The following arguments are optional:
     */
    document?: pulumi.Input<string>;
    /**
     * Name of the traffic policy.
     */
    name?: pulumi.Input<string>;
    /**
     * DNS type of the resource record sets that Amazon Route 53 creates when you use a traffic policy to create a traffic policy instance.
     */
    type?: pulumi.Input<string>;
    /**
     * Version number of the traffic policy. This value is automatically incremented by AWS after each update of this resource.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a TrafficPolicy resource.
 */
export interface TrafficPolicyArgs {
    /**
     * Comment for the traffic policy.
     */
    comment?: pulumi.Input<string>;
    /**
     * Policy document. This is a JSON formatted string. For more information about building Route53 traffic policy documents, see the [AWS Route53 Traffic Policy document format](https://docs.aws.amazon.com/Route53/latest/APIReference/api-policies-traffic-policy-document-format.html)
     *
     * The following arguments are optional:
     */
    document: pulumi.Input<string>;
    /**
     * Name of the traffic policy.
     */
    name?: pulumi.Input<string>;
}
