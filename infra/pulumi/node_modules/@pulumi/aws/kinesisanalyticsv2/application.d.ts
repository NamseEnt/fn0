import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Kinesis Analytics v2 Application.
 * This resource can be used to manage both Kinesis Data Analytics for SQL applications and Kinesis Data Analytics for Apache Flink applications.
 *
 * > **Note:** Kinesis Data Analytics for SQL applications created using this resource cannot currently be viewed in the AWS Console. To manage Kinesis Data Analytics for SQL applications that can also be viewed in the AWS Console, use the `aws.kinesis.AnalyticsApplication` resource.
 *
 * ## Example Usage
 *
 * ### Apache Flink Application
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example-flink-application"});
 * const exampleBucketObjectv2 = new aws.s3.BucketObjectv2("example", {
 *     bucket: example.id,
 *     key: "example-flink-application",
 *     source: new pulumi.asset.FileAsset("flink-app.jar"),
 * });
 * const exampleApplication = new aws.kinesisanalyticsv2.Application("example", {
 *     name: "example-flink-application",
 *     runtimeEnvironment: "FLINK-1_8",
 *     serviceExecutionRole: exampleAwsIamRole.arn,
 *     applicationConfiguration: {
 *         applicationCodeConfiguration: {
 *             codeContent: {
 *                 s3ContentLocation: {
 *                     bucketArn: example.arn,
 *                     fileKey: exampleBucketObjectv2.key,
 *                 },
 *             },
 *             codeContentType: "ZIPFILE",
 *         },
 *         environmentProperties: {
 *             propertyGroups: [
 *                 {
 *                     propertyGroupId: "PROPERTY-GROUP-1",
 *                     propertyMap: {
 *                         Key1: "Value1",
 *                     },
 *                 },
 *                 {
 *                     propertyGroupId: "PROPERTY-GROUP-2",
 *                     propertyMap: {
 *                         KeyA: "ValueA",
 *                         KeyB: "ValueB",
 *                     },
 *                 },
 *             ],
 *         },
 *         flinkApplicationConfiguration: {
 *             checkpointConfiguration: {
 *                 configurationType: "DEFAULT",
 *             },
 *             monitoringConfiguration: {
 *                 configurationType: "CUSTOM",
 *                 logLevel: "DEBUG",
 *                 metricsLevel: "TASK",
 *             },
 *             parallelismConfiguration: {
 *                 autoScalingEnabled: true,
 *                 configurationType: "CUSTOM",
 *                 parallelism: 10,
 *                 parallelismPerKpu: 4,
 *             },
 *         },
 *     },
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ### SQL Application
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {name: "example-sql-application"});
 * const exampleLogStream = new aws.cloudwatch.LogStream("example", {
 *     name: "example-sql-application",
 *     logGroupName: example.name,
 * });
 * const exampleApplication = new aws.kinesisanalyticsv2.Application("example", {
 *     name: "example-sql-application",
 *     runtimeEnvironment: "SQL-1_0",
 *     serviceExecutionRole: exampleAwsIamRole.arn,
 *     applicationConfiguration: {
 *         applicationCodeConfiguration: {
 *             codeContent: {
 *                 textContent: "SELECT 1;\n",
 *             },
 *             codeContentType: "PLAINTEXT",
 *         },
 *         sqlApplicationConfiguration: {
 *             input: {
 *                 namePrefix: "PREFIX_1",
 *                 inputParallelism: {
 *                     count: 3,
 *                 },
 *                 inputSchema: {
 *                     recordColumns: [
 *                         {
 *                             name: "COLUMN_1",
 *                             sqlType: "VARCHAR(8)",
 *                             mapping: "MAPPING-1",
 *                         },
 *                         {
 *                             name: "COLUMN_2",
 *                             sqlType: "DOUBLE",
 *                         },
 *                     ],
 *                     recordEncoding: "UTF-8",
 *                     recordFormat: {
 *                         recordFormatType: "CSV",
 *                         mappingParameters: {
 *                             csvMappingParameters: {
 *                                 recordColumnDelimiter: ",",
 *                                 recordRowDelimiter: "\n",
 *                             },
 *                         },
 *                     },
 *                 },
 *                 kinesisStreamsInput: {
 *                     resourceArn: exampleAwsKinesisStream.arn,
 *                 },
 *             },
 *             outputs: [
 *                 {
 *                     name: "OUTPUT_1",
 *                     destinationSchema: {
 *                         recordFormatType: "JSON",
 *                     },
 *                     lambdaOutput: {
 *                         resourceArn: exampleAwsLambdaFunction.arn,
 *                     },
 *                 },
 *                 {
 *                     name: "OUTPUT_2",
 *                     destinationSchema: {
 *                         recordFormatType: "CSV",
 *                     },
 *                     kinesisFirehoseOutput: {
 *                         resourceArn: exampleAwsKinesisFirehoseDeliveryStream.arn,
 *                     },
 *                 },
 *             ],
 *             referenceDataSource: {
 *                 tableName: "TABLE-1",
 *                 referenceSchema: {
 *                     recordColumns: [{
 *                         name: "COLUMN_1",
 *                         sqlType: "INTEGER",
 *                     }],
 *                     recordFormat: {
 *                         recordFormatType: "JSON",
 *                         mappingParameters: {
 *                             jsonMappingParameters: {
 *                                 recordRowPath: "$",
 *                             },
 *                         },
 *                     },
 *                 },
 *                 s3ReferenceDataSource: {
 *                     bucketArn: exampleAwsS3Bucket.arn,
 *                     fileKey: "KEY-1",
 *                 },
 *             },
 *         },
 *     },
 *     cloudwatchLoggingOptions: {
 *         logStreamArn: exampleLogStream.arn,
 *     },
 * });
 * ```
 *
 * ### VPC Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example-flink-application"});
 * const exampleBucketObjectv2 = new aws.s3.BucketObjectv2("example", {
 *     bucket: example.id,
 *     key: "example-flink-application",
 *     source: new pulumi.asset.FileAsset("flink-app.jar"),
 * });
 * const exampleApplication = new aws.kinesisanalyticsv2.Application("example", {
 *     name: "example-flink-application",
 *     runtimeEnvironment: "FLINK-1_8",
 *     serviceExecutionRole: exampleAwsIamRole.arn,
 *     applicationConfiguration: {
 *         applicationCodeConfiguration: {
 *             codeContent: {
 *                 s3ContentLocation: {
 *                     bucketArn: example.arn,
 *                     fileKey: exampleBucketObjectv2.key,
 *                 },
 *             },
 *             codeContentType: "ZIPFILE",
 *         },
 *         vpcConfiguration: {
 *             securityGroupIds: [
 *                 exampleAwsSecurityGroup[0].id,
 *                 exampleAwsSecurityGroup[1].id,
 *             ],
 *             subnetIds: [exampleAwsSubnet.id],
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_kinesisanalyticsv2_application` using the application ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:kinesisanalyticsv2/application:Application example arn:aws:kinesisanalytics:us-west-2:123456789012:application/example-sql-application
 * ```
 */
export declare class Application extends pulumi.CustomResource {
    /**
     * Get an existing Application resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationState, opts?: pulumi.CustomResourceOptions): Application;
    /**
     * Returns true if the given object is an instance of Application.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Application;
    /**
     * The application's configuration
     */
    readonly applicationConfiguration: pulumi.Output<outputs.kinesisanalyticsv2.ApplicationApplicationConfiguration>;
    /**
     * The application's mode. Valid values are `STREAMING`, `INTERACTIVE`.
     */
    readonly applicationMode: pulumi.Output<string>;
    /**
     * The ARN of the application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A CloudWatch log stream to monitor application configuration errors.
     */
    readonly cloudwatchLoggingOptions: pulumi.Output<outputs.kinesisanalyticsv2.ApplicationCloudwatchLoggingOptions | undefined>;
    /**
     * The current timestamp when the application was created.
     */
    readonly createTimestamp: pulumi.Output<string>;
    /**
     * A summary description of the application.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Whether to force stop an unresponsive Flink-based application.
     */
    readonly forceStop: pulumi.Output<boolean | undefined>;
    /**
     * The current timestamp when the application was last updated.
     */
    readonly lastUpdateTimestamp: pulumi.Output<string>;
    /**
     * The name of the application.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The runtime environment for the application. Valid values: `SQL-1_0`, `FLINK-1_6`, `FLINK-1_8`, `FLINK-1_11`, `FLINK-1_13`, `FLINK-1_15`, `FLINK-1_18`, `FLINK-1_19`.
     */
    readonly runtimeEnvironment: pulumi.Output<string>;
    /**
     * The ARN of the IAM role used by the application to access Kinesis data streams, Kinesis Data Firehose delivery streams, Amazon S3 objects, and other external resources.
     */
    readonly serviceExecutionRole: pulumi.Output<string>;
    /**
     * Whether to start or stop the application.
     */
    readonly startApplication: pulumi.Output<boolean | undefined>;
    /**
     * The status of the application.
     */
    readonly status: pulumi.Output<string>;
    /**
     * A map of tags to assign to the application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The current application version. Kinesis Data Analytics updates the `versionId` each time the application is updated.
     */
    readonly versionId: pulumi.Output<number>;
    /**
     * Create a Application resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Application resources.
 */
export interface ApplicationState {
    /**
     * The application's configuration
     */
    applicationConfiguration?: pulumi.Input<inputs.kinesisanalyticsv2.ApplicationApplicationConfiguration>;
    /**
     * The application's mode. Valid values are `STREAMING`, `INTERACTIVE`.
     */
    applicationMode?: pulumi.Input<string>;
    /**
     * The ARN of the application.
     */
    arn?: pulumi.Input<string>;
    /**
     * A CloudWatch log stream to monitor application configuration errors.
     */
    cloudwatchLoggingOptions?: pulumi.Input<inputs.kinesisanalyticsv2.ApplicationCloudwatchLoggingOptions>;
    /**
     * The current timestamp when the application was created.
     */
    createTimestamp?: pulumi.Input<string>;
    /**
     * A summary description of the application.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether to force stop an unresponsive Flink-based application.
     */
    forceStop?: pulumi.Input<boolean>;
    /**
     * The current timestamp when the application was last updated.
     */
    lastUpdateTimestamp?: pulumi.Input<string>;
    /**
     * The name of the application.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The runtime environment for the application. Valid values: `SQL-1_0`, `FLINK-1_6`, `FLINK-1_8`, `FLINK-1_11`, `FLINK-1_13`, `FLINK-1_15`, `FLINK-1_18`, `FLINK-1_19`.
     */
    runtimeEnvironment?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role used by the application to access Kinesis data streams, Kinesis Data Firehose delivery streams, Amazon S3 objects, and other external resources.
     */
    serviceExecutionRole?: pulumi.Input<string>;
    /**
     * Whether to start or stop the application.
     */
    startApplication?: pulumi.Input<boolean>;
    /**
     * The status of the application.
     */
    status?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The current application version. Kinesis Data Analytics updates the `versionId` each time the application is updated.
     */
    versionId?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Application resource.
 */
export interface ApplicationArgs {
    /**
     * The application's configuration
     */
    applicationConfiguration?: pulumi.Input<inputs.kinesisanalyticsv2.ApplicationApplicationConfiguration>;
    /**
     * The application's mode. Valid values are `STREAMING`, `INTERACTIVE`.
     */
    applicationMode?: pulumi.Input<string>;
    /**
     * A CloudWatch log stream to monitor application configuration errors.
     */
    cloudwatchLoggingOptions?: pulumi.Input<inputs.kinesisanalyticsv2.ApplicationCloudwatchLoggingOptions>;
    /**
     * A summary description of the application.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether to force stop an unresponsive Flink-based application.
     */
    forceStop?: pulumi.Input<boolean>;
    /**
     * The name of the application.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The runtime environment for the application. Valid values: `SQL-1_0`, `FLINK-1_6`, `FLINK-1_8`, `FLINK-1_11`, `FLINK-1_13`, `FLINK-1_15`, `FLINK-1_18`, `FLINK-1_19`.
     */
    runtimeEnvironment: pulumi.Input<string>;
    /**
     * The ARN of the IAM role used by the application to access Kinesis data streams, Kinesis Data Firehose delivery streams, Amazon S3 objects, and other external resources.
     */
    serviceExecutionRole: pulumi.Input<string>;
    /**
     * Whether to start or stop the application.
     */
    startApplication?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
