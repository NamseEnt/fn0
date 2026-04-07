import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudWatch Log Data Protection Policy resource.
 *
 * Read more about protecting sensitive user data in the [User Guide](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {name: "example"});
 * const exampleBucket = new aws.s3.Bucket("example", {bucket: "example"});
 * const exampleLogDataProtectionPolicy = new aws.cloudwatch.LogDataProtectionPolicy("example", {
 *     logGroupName: example.name,
 *     policyDocument: pulumi.jsonStringify({
 *         Name: "Example",
 *         Version: "2021-06-01",
 *         Statement: [
 *             {
 *                 Sid: "Audit",
 *                 DataIdentifier: ["arn:aws:dataprotection::aws:data-identifier/EmailAddress"],
 *                 Operation: {
 *                     Audit: {
 *                         FindingsDestination: {
 *                             S3: {
 *                                 Bucket: exampleBucket.bucket,
 *                             },
 *                         },
 *                     },
 *                 },
 *             },
 *             {
 *                 Sid: "Redact",
 *                 DataIdentifier: ["arn:aws:dataprotection::aws:data-identifier/EmailAddress"],
 *                 Operation: {
 *                     Deidentify: {
 *                         MaskConfig: {},
 *                     },
 *                 },
 *             },
 *         ],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import this resource using the `log_group_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logDataProtectionPolicy:LogDataProtectionPolicy example my-log-group
 * ```
 */
export declare class LogDataProtectionPolicy extends pulumi.CustomResource {
    /**
     * Get an existing LogDataProtectionPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogDataProtectionPolicyState, opts?: pulumi.CustomResourceOptions): LogDataProtectionPolicy;
    /**
     * Returns true if the given object is an instance of LogDataProtectionPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogDataProtectionPolicy;
    /**
     * The name of the log group under which the log stream is to be created.
     */
    readonly logGroupName: pulumi.Output<string>;
    /**
     * Specifies the data protection policy in JSON. Read more at [Data protection policy syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data-start.html#mask-sensitive-log-data-policysyntax).
     */
    readonly policyDocument: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LogDataProtectionPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogDataProtectionPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogDataProtectionPolicy resources.
 */
export interface LogDataProtectionPolicyState {
    /**
     * The name of the log group under which the log stream is to be created.
     */
    logGroupName?: pulumi.Input<string>;
    /**
     * Specifies the data protection policy in JSON. Read more at [Data protection policy syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data-start.html#mask-sensitive-log-data-policysyntax).
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogDataProtectionPolicy resource.
 */
export interface LogDataProtectionPolicyArgs {
    /**
     * The name of the log group under which the log stream is to be created.
     */
    logGroupName: pulumi.Input<string>;
    /**
     * Specifies the data protection policy in JSON. Read more at [Data protection policy syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data-start.html#mask-sensitive-log-data-policysyntax).
     */
    policyDocument: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
