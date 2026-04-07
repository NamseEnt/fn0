import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Cognito IDP (Identity Provider) Log Delivery Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage with CloudWatch Logs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {name: "example"});
 * const exampleLogGroup = new aws.cloudwatch.LogGroup("example", {name: "example"});
 * const exampleLogDeliveryConfiguration = new aws.cognito.LogDeliveryConfiguration("example", {
 *     userPoolId: example.id,
 *     logConfigurations: [{
 *         eventSource: "userNotification",
 *         logLevel: "ERROR",
 *         cloudWatchLogsConfiguration: {
 *             logGroupArn: exampleLogGroup.arn,
 *         },
 *     }],
 * });
 * ```
 *
 * ### Multiple Log Configurations with Different Destinations
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {name: "example"});
 * const exampleLogGroup = new aws.cloudwatch.LogGroup("example", {name: "example"});
 * const exampleBucket = new aws.s3.Bucket("example", {
 *     bucket: "example-bucket",
 *     forceDestroy: true,
 * });
 * const firehose = new aws.iam.Role("firehose", {
 *     name: "firehose-role",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: "firehose.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const firehoseRolePolicy = new aws.iam.RolePolicy("firehose", {
 *     name: "firehose-policy",
 *     role: firehose.id,
 *     policy: pulumi.jsonStringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Action: [
 *                 "s3:AbortMultipartUpload",
 *                 "s3:GetBucketLocation",
 *                 "s3:GetObject",
 *                 "s3:ListBucket",
 *                 "s3:ListBucketMultipartUploads",
 *                 "s3:PutObject",
 *             ],
 *             Resource: [
 *                 exampleBucket.arn,
 *                 pulumi.interpolate`${exampleBucket.arn}/*`,
 *             ],
 *         }],
 *     }),
 * });
 * const exampleFirehoseDeliveryStream = new aws.kinesis.FirehoseDeliveryStream("example", {
 *     name: "example-stream",
 *     destination: "extended_s3",
 *     extendedS3Configuration: {
 *         roleArn: firehose.arn,
 *         bucketArn: exampleBucket.arn,
 *     },
 * });
 * const exampleLogDeliveryConfiguration = new aws.cognito.LogDeliveryConfiguration("example", {
 *     userPoolId: example.id,
 *     logConfigurations: [
 *         {
 *             eventSource: "userNotification",
 *             logLevel: "INFO",
 *             cloudWatchLogsConfiguration: {
 *                 logGroupArn: exampleLogGroup.arn,
 *             },
 *         },
 *         {
 *             eventSource: "userAuthEvents",
 *             logLevel: "ERROR",
 *             firehoseConfiguration: {
 *                 streamArn: exampleFirehoseDeliveryStream.arn,
 *             },
 *         },
 *     ],
 * });
 * ```
 *
 * ### S3 Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {name: "example"});
 * const exampleBucket = new aws.s3.Bucket("example", {
 *     bucket: "example-bucket",
 *     forceDestroy: true,
 * });
 * const exampleLogDeliveryConfiguration = new aws.cognito.LogDeliveryConfiguration("example", {
 *     userPoolId: example.id,
 *     logConfigurations: [{
 *         eventSource: "userNotification",
 *         logLevel: "ERROR",
 *         s3Configuration: {
 *             bucketArn: exampleBucket.arn,
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `user_pool_id` (String) ID of the Cognito User Pool.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import Cognito IDP (Identity Provider) Log Delivery Configuration using the `user_pool_id`. For example:
 *
 * console
 *
 * % pulumi import aws_cognito_log_delivery_configuration.example us-west-2_example123
 */
export declare class LogDeliveryConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing LogDeliveryConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogDeliveryConfigurationState, opts?: pulumi.CustomResourceOptions): LogDeliveryConfiguration;
    /**
     * Returns true if the given object is an instance of LogDeliveryConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogDeliveryConfiguration;
    /**
     * Configuration block for log delivery. At least one configuration block is required. See Log Configurations below.
     */
    readonly logConfigurations: pulumi.Output<outputs.cognito.LogDeliveryConfigurationLogConfiguration[] | undefined>;
    /**
     * The AWS region.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the user pool for which to configure log delivery.
     *
     * The following arguments are optional:
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * Create a LogDeliveryConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogDeliveryConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogDeliveryConfiguration resources.
 */
export interface LogDeliveryConfigurationState {
    /**
     * Configuration block for log delivery. At least one configuration block is required. See Log Configurations below.
     */
    logConfigurations?: pulumi.Input<pulumi.Input<inputs.cognito.LogDeliveryConfigurationLogConfiguration>[]>;
    /**
     * The AWS region.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the user pool for which to configure log delivery.
     *
     * The following arguments are optional:
     */
    userPoolId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogDeliveryConfiguration resource.
 */
export interface LogDeliveryConfigurationArgs {
    /**
     * Configuration block for log delivery. At least one configuration block is required. See Log Configurations below.
     */
    logConfigurations?: pulumi.Input<pulumi.Input<inputs.cognito.LogDeliveryConfigurationLogConfiguration>[]>;
    /**
     * The AWS region.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the user pool for which to configure log delivery.
     *
     * The following arguments are optional:
     */
    userPoolId: pulumi.Input<string>;
}
