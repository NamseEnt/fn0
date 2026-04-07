import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS IVS (Interactive Video) Chat Logging Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage - Logging to CloudWatch
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {});
 * const exampleLoggingConfiguration = new aws.ivschat.LoggingConfiguration("example", {destinationConfiguration: {
 *     cloudwatchLogs: {
 *         logGroupName: example.name,
 *     },
 * }});
 * ```
 *
 * ### Basic Usage - Logging to Kinesis Firehose with Extended S3
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleBucket = new aws.s3.Bucket("example", {bucketPrefix: "tf-ivschat-logging-bucket"});
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["firehose.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "firehose_example_role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const example = new aws.kinesis.FirehoseDeliveryStream("example", {
 *     name: "pulumi-kinesis-firehose-extended-s3-example-stream",
 *     destination: "extended_s3",
 *     extendedS3Configuration: {
 *         roleArn: exampleRole.arn,
 *         bucketArn: exampleBucket.arn,
 *     },
 *     tags: {
 *         LogDeliveryEnabled: "true",
 *     },
 * });
 * const exampleBucketAcl = new aws.s3.BucketAcl("example", {
 *     bucket: exampleBucket.id,
 *     acl: "private",
 * });
 * const exampleLoggingConfiguration = new aws.ivschat.LoggingConfiguration("example", {destinationConfiguration: {
 *     firehose: {
 *         deliveryStreamName: example.name,
 *     },
 * }});
 * ```
 *
 * ### Basic Usage - Logging to S3
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {
 *     bucketName: "tf-ivschat-logging",
 *     forceDestroy: true,
 * });
 * const exampleLoggingConfiguration = new aws.ivschat.LoggingConfiguration("example", {destinationConfiguration: {
 *     s3: {
 *         bucketName: example.id,
 *     },
 * }});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the IVS Chat logging configuration.
 *
 * Using `pulumi import`, import IVS (Interactive Video) Chat Logging Configuration using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_ivschat_logging_configuration.example arn:aws:ivschat:us-west-2:326937407773:logging-configuration/MMUQc8wcqZmC
 */
export declare class LoggingConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing LoggingConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LoggingConfigurationState, opts?: pulumi.CustomResourceOptions): LoggingConfiguration;
    /**
     * Returns true if the given object is an instance of LoggingConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LoggingConfiguration;
    /**
     * ARN of the Logging Configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Object containing destination configuration for where chat activity will be logged. This object must contain exactly one of the following children arguments:
     */
    readonly destinationConfiguration: pulumi.Output<outputs.ivschat.LoggingConfigurationDestinationConfiguration | undefined>;
    /**
     * Logging Configuration name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the Logging Configuration.
     */
    readonly state: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a LoggingConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: LoggingConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LoggingConfiguration resources.
 */
export interface LoggingConfigurationState {
    /**
     * ARN of the Logging Configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Object containing destination configuration for where chat activity will be logged. This object must contain exactly one of the following children arguments:
     */
    destinationConfiguration?: pulumi.Input<inputs.ivschat.LoggingConfigurationDestinationConfiguration>;
    /**
     * Logging Configuration name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the Logging Configuration.
     */
    state?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a LoggingConfiguration resource.
 */
export interface LoggingConfigurationArgs {
    /**
     * Object containing destination configuration for where chat activity will be logged. This object must contain exactly one of the following children arguments:
     */
    destinationConfiguration?: pulumi.Input<inputs.ivschat.LoggingConfigurationDestinationConfiguration>;
    /**
     * Logging Configuration name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
