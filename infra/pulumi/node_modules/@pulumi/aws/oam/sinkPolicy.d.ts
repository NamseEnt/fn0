import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CloudWatch Observability Access Manager Sink Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.oam.Sink("example", {name: "ExampleSink"});
 * const exampleSinkPolicy = new aws.oam.SinkPolicy("example", {
 *     sinkIdentifier: example.arn,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: [
 *                 "oam:CreateLink",
 *                 "oam:UpdateLink",
 *             ],
 *             Effect: "Allow",
 *             Resource: "*",
 *             Principal: {
 *                 AWS: [
 *                     "1111111111111",
 *                     "222222222222",
 *                 ],
 *             },
 *             Condition: {
 *                 "ForAllValues:StringEquals": {
 *                     "oam:ResourceTypes": [
 *                         "AWS::CloudWatch::Metric",
 *                         "AWS::Logs::LogGroup",
 *                     ],
 *                 },
 *             },
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Observability Access Manager Sink Policy using the `sink_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:oam/sinkPolicy:SinkPolicy example arn:aws:oam:us-west-2:123456789012:sink/sink-id
 * ```
 */
export declare class SinkPolicy extends pulumi.CustomResource {
    /**
     * Get an existing SinkPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SinkPolicyState, opts?: pulumi.CustomResourceOptions): SinkPolicy;
    /**
     * Returns true if the given object is an instance of SinkPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SinkPolicy;
    /**
     * ARN of the Sink.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * JSON policy to use. If you are updating an existing policy, the entire existing policy is replaced by what you specify here.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID string that AWS generated as part of the sink ARN.
     */
    readonly sinkId: pulumi.Output<string>;
    /**
     * ARN of the sink to attach this policy to.
     */
    readonly sinkIdentifier: pulumi.Output<string>;
    /**
     * Create a SinkPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SinkPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SinkPolicy resources.
 */
export interface SinkPolicyState {
    /**
     * ARN of the Sink.
     */
    arn?: pulumi.Input<string>;
    /**
     * JSON policy to use. If you are updating an existing policy, the entire existing policy is replaced by what you specify here.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID string that AWS generated as part of the sink ARN.
     */
    sinkId?: pulumi.Input<string>;
    /**
     * ARN of the sink to attach this policy to.
     */
    sinkIdentifier?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SinkPolicy resource.
 */
export interface SinkPolicyArgs {
    /**
     * JSON policy to use. If you are updating an existing policy, the entire existing policy is replaced by what you specify here.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the sink to attach this policy to.
     */
    sinkIdentifier: pulumi.Input<string>;
}
