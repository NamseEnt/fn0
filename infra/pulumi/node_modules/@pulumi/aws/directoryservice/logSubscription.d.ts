import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Log subscription for AWS Directory Service that pushes logs to cloudwatch.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {
 *     name: `/aws/directoryservice/${exampleAwsDirectoryServiceDirectory.id}`,
 *     retentionInDays: 14,
 * });
 * const ad_log_policy = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         actions: [
 *             "logs:CreateLogStream",
 *             "logs:PutLogEvents",
 *         ],
 *         principals: [{
 *             identifiers: ["ds.amazonaws.com"],
 *             type: "Service",
 *         }],
 *         resources: [pulumi.interpolate`${example.arn}:*`],
 *         effect: "Allow",
 *     }],
 * });
 * const ad_log_policyLogResourcePolicy = new aws.cloudwatch.LogResourcePolicy("ad-log-policy", {
 *     policyDocument: ad_log_policy.apply(ad_log_policy => ad_log_policy.json),
 *     policyName: "ad-log-policy",
 * });
 * const exampleLogSubscription = new aws.directoryservice.LogSubscription("example", {
 *     directoryId: exampleAwsDirectoryServiceDirectory.id,
 *     logGroupName: example.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Directory Service Log Subscriptions using the directory id. For example:
 *
 * ```sh
 * $ pulumi import aws:directoryservice/logSubscription:LogSubscription msad d-1234567890
 * ```
 */
export declare class LogSubscription extends pulumi.CustomResource {
    /**
     * Get an existing LogSubscription resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogSubscriptionState, opts?: pulumi.CustomResourceOptions): LogSubscription;
    /**
     * Returns true if the given object is an instance of LogSubscription.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogSubscription;
    /**
     * ID of directory.
     */
    readonly directoryId: pulumi.Output<string>;
    /**
     * Name of the cloudwatch log group to which the logs should be published. The log group should be already created and the directory service principal should be provided with required permission to create stream and publish logs. Changing this value would delete the current subscription and create a new one. A directory can only have one log subscription at a time.
     */
    readonly logGroupName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LogSubscription resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogSubscriptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogSubscription resources.
 */
export interface LogSubscriptionState {
    /**
     * ID of directory.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Name of the cloudwatch log group to which the logs should be published. The log group should be already created and the directory service principal should be provided with required permission to create stream and publish logs. Changing this value would delete the current subscription and create a new one. A directory can only have one log subscription at a time.
     */
    logGroupName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogSubscription resource.
 */
export interface LogSubscriptionArgs {
    /**
     * ID of directory.
     */
    directoryId: pulumi.Input<string>;
    /**
     * Name of the cloudwatch log group to which the logs should be published. The log group should be already created and the directory service principal should be provided with required permission to create stream and publish logs. Changing this value would delete the current subscription and create a new one. A directory can only have one log subscription at a time.
     */
    logGroupName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
