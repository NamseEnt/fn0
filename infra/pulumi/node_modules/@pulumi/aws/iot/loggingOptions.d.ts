import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage [default logging options](https://docs.aws.amazon.com/iot/latest/developerguide/configure-logging.html#configure-logging-console).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iot.LoggingOptions("example", {
 *     defaultLogLevel: "WARN",
 *     roleArn: exampleAwsIamRole.arn,
 * });
 * ```
 */
export declare class LoggingOptions extends pulumi.CustomResource {
    /**
     * Get an existing LoggingOptions resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LoggingOptionsState, opts?: pulumi.CustomResourceOptions): LoggingOptions;
    /**
     * Returns true if the given object is an instance of LoggingOptions.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LoggingOptions;
    /**
     * The default logging level. Valid Values: `"DEBUG"`, `"INFO"`, `"ERROR"`, `"WARN"`, `"DISABLED"`.
     */
    readonly defaultLogLevel: pulumi.Output<string>;
    /**
     * If `true` all logs are disabled. The default is `false`.
     */
    readonly disableAllLogs: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the role that allows IoT to write to Cloudwatch logs.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Create a LoggingOptions resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LoggingOptionsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LoggingOptions resources.
 */
export interface LoggingOptionsState {
    /**
     * The default logging level. Valid Values: `"DEBUG"`, `"INFO"`, `"ERROR"`, `"WARN"`, `"DISABLED"`.
     */
    defaultLogLevel?: pulumi.Input<string>;
    /**
     * If `true` all logs are disabled. The default is `false`.
     */
    disableAllLogs?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the role that allows IoT to write to Cloudwatch logs.
     */
    roleArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LoggingOptions resource.
 */
export interface LoggingOptionsArgs {
    /**
     * The default logging level. Valid Values: `"DEBUG"`, `"INFO"`, `"ERROR"`, `"WARN"`, `"DISABLED"`.
     */
    defaultLogLevel: pulumi.Input<string>;
    /**
     * If `true` all logs are disabled. The default is `false`.
     */
    disableAllLogs?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the role that allows IoT to write to Cloudwatch logs.
     */
    roleArn: pulumi.Input<string>;
}
