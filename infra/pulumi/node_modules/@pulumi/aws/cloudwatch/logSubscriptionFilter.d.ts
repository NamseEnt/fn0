import * as pulumi from "@pulumi/pulumi";
import { LogGroup } from "./index";
/**
 * Provides a CloudWatch Logs subscription filter resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testLambdafunctionLogfilter = new aws.cloudwatch.LogSubscriptionFilter("test_lambdafunction_logfilter", {
 *     name: "test_lambdafunction_logfilter",
 *     roleArn: iamForLambda.arn,
 *     logGroup: "/aws/lambda/example_lambda_name",
 *     filterPattern: "logtype test",
 *     destinationArn: testLogstream.arn,
 *     distribution: "Random",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Logs subscription filter using the log group name and subscription filter name separated by `|`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logSubscriptionFilter:LogSubscriptionFilter test_lambdafunction_logfilter "/aws/lambda/example_lambda_name|test_lambdafunction_logfilter"
 * ```
 */
export declare class LogSubscriptionFilter extends pulumi.CustomResource {
    /**
     * Get an existing LogSubscriptionFilter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogSubscriptionFilterState, opts?: pulumi.CustomResourceOptions): LogSubscriptionFilter;
    /**
     * Returns true if the given object is an instance of LogSubscriptionFilter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogSubscriptionFilter;
    /**
     * The ARN of the destination to deliver matching log events to. Kinesis stream or Lambda function ARN.
     */
    readonly destinationArn: pulumi.Output<string>;
    /**
     * The method used to distribute log data to the destination. By default log data is grouped by log stream, but the grouping can be set to random for a more even distribution. This property is only applicable when the destination is an Amazon Kinesis stream. Valid values are "Random" and "ByLogStream".
     */
    readonly distribution: pulumi.Output<string | undefined>;
    /**
     * A valid CloudWatch Logs filter pattern for subscribing to a filtered stream of log events. Use empty string `""` to match everything. For more information, see the [Amazon CloudWatch Logs User Guide](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/FilterAndPatternSyntax.html).
     */
    readonly filterPattern: pulumi.Output<string>;
    /**
     * The name of the log group to associate the subscription filter with
     */
    readonly logGroup: pulumi.Output<string>;
    /**
     * A name for the subscription filter
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of an IAM role that grants Amazon CloudWatch Logs permissions to deliver ingested log events to the destination. If you use Lambda as a destination, you should skip this argument and use `aws.lambda.Permission` resource for granting access from CloudWatch logs to the destination Lambda function.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Create a LogSubscriptionFilter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogSubscriptionFilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogSubscriptionFilter resources.
 */
export interface LogSubscriptionFilterState {
    /**
     * The ARN of the destination to deliver matching log events to. Kinesis stream or Lambda function ARN.
     */
    destinationArn?: pulumi.Input<string>;
    /**
     * The method used to distribute log data to the destination. By default log data is grouped by log stream, but the grouping can be set to random for a more even distribution. This property is only applicable when the destination is an Amazon Kinesis stream. Valid values are "Random" and "ByLogStream".
     */
    distribution?: pulumi.Input<string>;
    /**
     * A valid CloudWatch Logs filter pattern for subscribing to a filtered stream of log events. Use empty string `""` to match everything. For more information, see the [Amazon CloudWatch Logs User Guide](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/FilterAndPatternSyntax.html).
     */
    filterPattern?: pulumi.Input<string>;
    /**
     * The name of the log group to associate the subscription filter with
     */
    logGroup?: pulumi.Input<string | LogGroup>;
    /**
     * A name for the subscription filter
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of an IAM role that grants Amazon CloudWatch Logs permissions to deliver ingested log events to the destination. If you use Lambda as a destination, you should skip this argument and use `aws.lambda.Permission` resource for granting access from CloudWatch logs to the destination Lambda function.
     */
    roleArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogSubscriptionFilter resource.
 */
export interface LogSubscriptionFilterArgs {
    /**
     * The ARN of the destination to deliver matching log events to. Kinesis stream or Lambda function ARN.
     */
    destinationArn: pulumi.Input<string>;
    /**
     * The method used to distribute log data to the destination. By default log data is grouped by log stream, but the grouping can be set to random for a more even distribution. This property is only applicable when the destination is an Amazon Kinesis stream. Valid values are "Random" and "ByLogStream".
     */
    distribution?: pulumi.Input<string>;
    /**
     * A valid CloudWatch Logs filter pattern for subscribing to a filtered stream of log events. Use empty string `""` to match everything. For more information, see the [Amazon CloudWatch Logs User Guide](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/FilterAndPatternSyntax.html).
     */
    filterPattern: pulumi.Input<string>;
    /**
     * The name of the log group to associate the subscription filter with
     */
    logGroup: pulumi.Input<string | LogGroup>;
    /**
     * A name for the subscription filter
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of an IAM role that grants Amazon CloudWatch Logs permissions to deliver ingested log events to the destination. If you use Lambda as a destination, you should skip this argument and use `aws.lambda.Permission` resource for granting access from CloudWatch logs to the destination Lambda function.
     */
    roleArn?: pulumi.Input<string>;
}
