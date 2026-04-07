import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CloudWatch Logs Index Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {name: "example"});
 * const exampleLogIndexPolicy = new aws.cloudwatch.LogIndexPolicy("example", {
 *     logGroupName: example.name,
 *     policyDocument: JSON.stringify({
 *         Fields: ["eventName"],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Logs Index Policy using the `log_group_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logIndexPolicy:LogIndexPolicy example /aws/log/group/name
 * ```
 */
export declare class LogIndexPolicy extends pulumi.CustomResource {
    /**
     * Get an existing LogIndexPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogIndexPolicyState, opts?: pulumi.CustomResourceOptions): LogIndexPolicy;
    /**
     * Returns true if the given object is an instance of LogIndexPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogIndexPolicy;
    /**
     * Log group name to set the policy for.
     */
    readonly logGroupName: pulumi.Output<string>;
    /**
     * JSON policy document. This is a JSON formatted string.
     */
    readonly policyDocument: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LogIndexPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogIndexPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogIndexPolicy resources.
 */
export interface LogIndexPolicyState {
    /**
     * Log group name to set the policy for.
     */
    logGroupName?: pulumi.Input<string>;
    /**
     * JSON policy document. This is a JSON formatted string.
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogIndexPolicy resource.
 */
export interface LogIndexPolicyArgs {
    /**
     * Log group name to set the policy for.
     */
    logGroupName: pulumi.Input<string>;
    /**
     * JSON policy document. This is a JSON formatted string.
     */
    policyDocument: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
