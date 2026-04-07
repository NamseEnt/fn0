import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Rekognition Stream Processor.
 *
 * > This resource must be configured specifically for your use case, and not all options are compatible with one another. See [Stream Processor API documentation](https://docs.aws.amazon.com/rekognition/latest/APIReference/API_CreateStreamProcessor.html#rekognition-CreateStreamProcessor-request-Input) for configuration information.
 *
 * > Stream Processors configured for Face Recognition cannot have _any_ properties updated after the fact, and it will result in an AWS API error.
 *
 * ## Example Usage
 *
 * ### Label Detection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example-bucket"});
 * const exampleTopic = new aws.sns.Topic("example", {name: "example-topic"});
 * const exampleVideoStream = new aws.kinesis.VideoStream("example", {
 *     name: "example-kinesis-input",
 *     dataRetentionInHours: 1,
 *     deviceName: "kinesis-video-device-name",
 *     mediaType: "video/h264",
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example-role",
 *     inlinePolicies: [{
 *         name: "Rekognition-Access",
 *         policy: pulumi.jsonStringify({
 *             Version: "2012-10-17",
 *             Statement: [
 *                 {
 *                     Action: ["s3:PutObject"],
 *                     Effect: "Allow",
 *                     Resource: [pulumi.interpolate`${example.arn}/*`],
 *                 },
 *                 {
 *                     Action: ["sns:Publish"],
 *                     Effect: "Allow",
 *                     Resource: [exampleTopic.arn],
 *                 },
 *                 {
 *                     Action: [
 *                         "kinesis:Get*",
 *                         "kinesis:DescribeStreamSummary",
 *                     ],
 *                     Effect: "Allow",
 *                     Resource: [exampleVideoStream.arn],
 *                 },
 *             ],
 *         }),
 *     }],
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: "rekognition.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const exampleStreamProcessor = new aws.rekognition.StreamProcessor("example", {
 *     roleArn: exampleRole.arn,
 *     name: "example-processor",
 *     dataSharingPreference: {
 *         optIn: false,
 *     },
 *     output: {
 *         s3Destination: {
 *             bucket: example.bucket,
 *         },
 *     },
 *     settings: {
 *         connectedHome: {
 *             labels: [
 *                 "PERSON",
 *                 "PET",
 *             ],
 *         },
 *     },
 *     input: {
 *         kinesisVideoStream: {
 *             arn: exampleVideoStream.arn,
 *         },
 *     },
 *     notificationChannel: {
 *         snsTopicArn: exampleTopic.arn,
 *     },
 * });
 * ```
 *
 * ### Face Detection Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kinesis.VideoStream("example", {
 *     name: "example-kinesis-input",
 *     dataRetentionInHours: 1,
 *     deviceName: "kinesis-video-device-name",
 *     mediaType: "video/h264",
 * });
 * const exampleStream = new aws.kinesis.Stream("example", {
 *     name: "pulumi-kinesis-example",
 *     shardCount: 1,
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example-role",
 *     inlinePolicies: [{
 *         name: "Rekognition-Access",
 *         policy: pulumi.jsonStringify({
 *             Version: "2012-10-17",
 *             Statement: [
 *                 {
 *                     Action: [
 *                         "kinesis:Get*",
 *                         "kinesis:DescribeStreamSummary",
 *                     ],
 *                     Effect: "Allow",
 *                     Resource: [example.arn],
 *                 },
 *                 {
 *                     Action: ["kinesis:PutRecord"],
 *                     Effect: "Allow",
 *                     Resource: [exampleStream.arn],
 *                 },
 *             ],
 *         }),
 *     }],
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: "rekognition.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const exampleCollection = new aws.rekognition.Collection("example", {collectionId: "example-collection"});
 * const exampleStreamProcessor = new aws.rekognition.StreamProcessor("example", {
 *     roleArn: exampleRole.arn,
 *     name: "example-processor",
 *     dataSharingPreference: {
 *         optIn: false,
 *     },
 *     regionsOfInterests: [{
 *         polygons: [
 *             {
 *                 x: 0.5,
 *                 y: 0.5,
 *             },
 *             {
 *                 x: 0.5,
 *                 y: 0.5,
 *             },
 *             {
 *                 x: 0.5,
 *                 y: 0.5,
 *             },
 *         ],
 *     }],
 *     input: {
 *         kinesisVideoStream: {
 *             arn: example.arn,
 *         },
 *     },
 *     output: {
 *         kinesisDataStream: {
 *             arn: exampleStream.arn,
 *         },
 *     },
 *     settings: {
 *         faceSearch: {
 *             collectionId: exampleCollection.id,
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Rekognition Stream Processor using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:rekognition/streamProcessor:StreamProcessor example my-stream
 * ```
 */
export declare class StreamProcessor extends pulumi.CustomResource {
    /**
     * Get an existing StreamProcessor resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StreamProcessorState, opts?: pulumi.CustomResourceOptions): StreamProcessor;
    /**
     * Returns true if the given object is an instance of StreamProcessor.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StreamProcessor;
    /**
     * ARN of the Stream Processor.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * See `dataSharingPreference`.
     */
    readonly dataSharingPreference: pulumi.Output<outputs.rekognition.StreamProcessorDataSharingPreference | undefined>;
    /**
     * Input video stream. See `input`.
     */
    readonly input: pulumi.Output<outputs.rekognition.StreamProcessorInput | undefined>;
    /**
     * Optional parameter for label detection stream processors.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * The name of the Stream Processor.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The Amazon Simple Notification Service topic to which Amazon Rekognition publishes the completion status. See `notificationChannel`.
     */
    readonly notificationChannel: pulumi.Output<outputs.rekognition.StreamProcessorNotificationChannel | undefined>;
    /**
     * Kinesis data stream stream or Amazon S3 bucket location to which Amazon Rekognition Video puts the analysis results. See `output`.
     */
    readonly output: pulumi.Output<outputs.rekognition.StreamProcessorOutput | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies locations in the frames where Amazon Rekognition checks for objects or people. See `regionsOfInterest`.
     */
    readonly regionsOfInterests: pulumi.Output<outputs.rekognition.StreamProcessorRegionsOfInterest[] | undefined>;
    /**
     * The Amazon Resource Number (ARN) of the IAM role that allows access to the stream processor. The IAM role provides Rekognition read permissions for a Kinesis stream. It also provides write permissions to an Amazon S3 bucket and Amazon Simple Notification Service topic for a label detection stream processor. This is required for both face search and label detection stream processors.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Input parameters used in a streaming video analyzed by a stream processor. See `settings`.
     *
     * The following arguments are optional:
     */
    readonly settings: pulumi.Output<outputs.rekognition.StreamProcessorSettings | undefined>;
    /**
     * (**Deprecated**) ARN of the Stream Processor.
     * Use `arn` instead.
     *
     * @deprecated Use 'arn' instead. This attribute will be removed in a future version of the provider.
     */
    readonly streamProcessorArn: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.rekognition.StreamProcessorTimeouts | undefined>;
    /**
     * Create a StreamProcessor resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StreamProcessorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StreamProcessor resources.
 */
export interface StreamProcessorState {
    /**
     * ARN of the Stream Processor.
     */
    arn?: pulumi.Input<string>;
    /**
     * See `dataSharingPreference`.
     */
    dataSharingPreference?: pulumi.Input<inputs.rekognition.StreamProcessorDataSharingPreference>;
    /**
     * Input video stream. See `input`.
     */
    input?: pulumi.Input<inputs.rekognition.StreamProcessorInput>;
    /**
     * Optional parameter for label detection stream processors.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of the Stream Processor.
     */
    name?: pulumi.Input<string>;
    /**
     * The Amazon Simple Notification Service topic to which Amazon Rekognition publishes the completion status. See `notificationChannel`.
     */
    notificationChannel?: pulumi.Input<inputs.rekognition.StreamProcessorNotificationChannel>;
    /**
     * Kinesis data stream stream or Amazon S3 bucket location to which Amazon Rekognition Video puts the analysis results. See `output`.
     */
    output?: pulumi.Input<inputs.rekognition.StreamProcessorOutput>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies locations in the frames where Amazon Rekognition checks for objects or people. See `regionsOfInterest`.
     */
    regionsOfInterests?: pulumi.Input<pulumi.Input<inputs.rekognition.StreamProcessorRegionsOfInterest>[]>;
    /**
     * The Amazon Resource Number (ARN) of the IAM role that allows access to the stream processor. The IAM role provides Rekognition read permissions for a Kinesis stream. It also provides write permissions to an Amazon S3 bucket and Amazon Simple Notification Service topic for a label detection stream processor. This is required for both face search and label detection stream processors.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Input parameters used in a streaming video analyzed by a stream processor. See `settings`.
     *
     * The following arguments are optional:
     */
    settings?: pulumi.Input<inputs.rekognition.StreamProcessorSettings>;
    /**
     * (**Deprecated**) ARN of the Stream Processor.
     * Use `arn` instead.
     *
     * @deprecated Use 'arn' instead. This attribute will be removed in a future version of the provider.
     */
    streamProcessorArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.rekognition.StreamProcessorTimeouts>;
}
/**
 * The set of arguments for constructing a StreamProcessor resource.
 */
export interface StreamProcessorArgs {
    /**
     * See `dataSharingPreference`.
     */
    dataSharingPreference?: pulumi.Input<inputs.rekognition.StreamProcessorDataSharingPreference>;
    /**
     * Input video stream. See `input`.
     */
    input?: pulumi.Input<inputs.rekognition.StreamProcessorInput>;
    /**
     * Optional parameter for label detection stream processors.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of the Stream Processor.
     */
    name?: pulumi.Input<string>;
    /**
     * The Amazon Simple Notification Service topic to which Amazon Rekognition publishes the completion status. See `notificationChannel`.
     */
    notificationChannel?: pulumi.Input<inputs.rekognition.StreamProcessorNotificationChannel>;
    /**
     * Kinesis data stream stream or Amazon S3 bucket location to which Amazon Rekognition Video puts the analysis results. See `output`.
     */
    output?: pulumi.Input<inputs.rekognition.StreamProcessorOutput>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies locations in the frames where Amazon Rekognition checks for objects or people. See `regionsOfInterest`.
     */
    regionsOfInterests?: pulumi.Input<pulumi.Input<inputs.rekognition.StreamProcessorRegionsOfInterest>[]>;
    /**
     * The Amazon Resource Number (ARN) of the IAM role that allows access to the stream processor. The IAM role provides Rekognition read permissions for a Kinesis stream. It also provides write permissions to an Amazon S3 bucket and Amazon Simple Notification Service topic for a label detection stream processor. This is required for both face search and label detection stream processors.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Input parameters used in a streaming video analyzed by a stream processor. See `settings`.
     *
     * The following arguments are optional:
     */
    settings?: pulumi.Input<inputs.rekognition.StreamProcessorSettings>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.rekognition.StreamProcessorTimeouts>;
}
