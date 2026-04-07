import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon Connect Instance Storage Config resource. For more information see
 * [Amazon Connect: Getting Started](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-get-started.html)
 *
 * ## Example Usage
 *
 * ### Storage Config Kinesis Firehose Config
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.InstanceStorageConfig("example", {
 *     instanceId: exampleAwsConnectInstance.id,
 *     resourceType: "CONTACT_TRACE_RECORDS",
 *     storageConfig: {
 *         kinesisFirehoseConfig: {
 *             firehoseArn: exampleAwsKinesisFirehoseDeliveryStream.arn,
 *         },
 *         storageType: "KINESIS_FIREHOSE",
 *     },
 * });
 * ```
 *
 * ### Storage Config Kinesis Stream Config
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.InstanceStorageConfig("example", {
 *     instanceId: exampleAwsConnectInstance.id,
 *     resourceType: "CONTACT_TRACE_RECORDS",
 *     storageConfig: {
 *         kinesisStreamConfig: {
 *             streamArn: exampleAwsKinesisStream.arn,
 *         },
 *         storageType: "KINESIS_STREAM",
 *     },
 * });
 * ```
 *
 * ### Storage Config Kinesis Video Stream Config
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.InstanceStorageConfig("example", {
 *     instanceId: exampleAwsConnectInstance.id,
 *     resourceType: "MEDIA_STREAMS",
 *     storageConfig: {
 *         kinesisVideoStreamConfig: {
 *             prefix: "example",
 *             retentionPeriodHours: 3,
 *             encryptionConfig: {
 *                 encryptionType: "KMS",
 *                 keyId: exampleAwsKmsKey.arn,
 *             },
 *         },
 *         storageType: "KINESIS_VIDEO_STREAM",
 *     },
 * });
 * ```
 *
 * ### Storage Config S3 Config
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.InstanceStorageConfig("example", {
 *     instanceId: exampleAwsConnectInstance.id,
 *     resourceType: "CHAT_TRANSCRIPTS",
 *     storageConfig: {
 *         s3Config: {
 *             bucketName: exampleAwsS3Bucket.id,
 *             bucketPrefix: "example",
 *         },
 *         storageType: "S3",
 *     },
 * });
 * ```
 *
 * ### Storage Config S3 Config with Encryption Config
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.InstanceStorageConfig("example", {
 *     instanceId: exampleAwsConnectInstance.id,
 *     resourceType: "CHAT_TRANSCRIPTS",
 *     storageConfig: {
 *         s3Config: {
 *             bucketName: exampleAwsS3Bucket.id,
 *             bucketPrefix: "example",
 *             encryptionConfig: {
 *                 encryptionType: "KMS",
 *                 keyId: exampleAwsKmsKey.arn,
 *             },
 *         },
 *         storageType: "S3",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Connect Instance Storage Configs using the `instance_id`, `association_id`, and `resource_type` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:connect/instanceStorageConfig:InstanceStorageConfig example f1288a1f-6193-445a-b47e-af739b2:c1d4e5f6-1b3c-1b3c-1b3c-c1d4e5f6c1d4e5:CHAT_TRANSCRIPTS
 * ```
 */
export declare class InstanceStorageConfig extends pulumi.CustomResource {
    /**
     * Get an existing InstanceStorageConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceStorageConfigState, opts?: pulumi.CustomResourceOptions): InstanceStorageConfig;
    /**
     * Returns true if the given object is an instance of InstanceStorageConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceStorageConfig;
    /**
     * The existing association identifier that uniquely identifies the resource type and storage config for the given instance ID.
     */
    readonly associationId: pulumi.Output<string>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A valid resource type. Valid Values: `AGENT_EVENTS` | `ATTACHMENTS` | `CALL_RECORDINGS` | `CHAT_TRANSCRIPTS` | `CONTACT_EVALUATIONS` | `CONTACT_TRACE_RECORDS` | `EMAIL_MESSAGES` | `MEDIA_STREAMS` | `REAL_TIME_CONTACT_ANALYSIS_CHAT_SEGMENTS` | `REAL_TIME_CONTACT_ANALYSIS_SEGMENTS` | `REAL_TIME_CONTACT_ANALYSIS_VOICE_SEGMENTS` | `SCHEDULED_REPORTS` | `SCREEN_RECORDINGS`.
     */
    readonly resourceType: pulumi.Output<string>;
    /**
     * Specifies the storage configuration options for the Connect Instance. Documented below.
     */
    readonly storageConfig: pulumi.Output<outputs.connect.InstanceStorageConfigStorageConfig>;
    /**
     * Create a InstanceStorageConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceStorageConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceStorageConfig resources.
 */
export interface InstanceStorageConfigState {
    /**
     * The existing association identifier that uniquely identifies the resource type and storage config for the given instance ID.
     */
    associationId?: pulumi.Input<string>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A valid resource type. Valid Values: `AGENT_EVENTS` | `ATTACHMENTS` | `CALL_RECORDINGS` | `CHAT_TRANSCRIPTS` | `CONTACT_EVALUATIONS` | `CONTACT_TRACE_RECORDS` | `EMAIL_MESSAGES` | `MEDIA_STREAMS` | `REAL_TIME_CONTACT_ANALYSIS_CHAT_SEGMENTS` | `REAL_TIME_CONTACT_ANALYSIS_SEGMENTS` | `REAL_TIME_CONTACT_ANALYSIS_VOICE_SEGMENTS` | `SCHEDULED_REPORTS` | `SCREEN_RECORDINGS`.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Specifies the storage configuration options for the Connect Instance. Documented below.
     */
    storageConfig?: pulumi.Input<inputs.connect.InstanceStorageConfigStorageConfig>;
}
/**
 * The set of arguments for constructing a InstanceStorageConfig resource.
 */
export interface InstanceStorageConfigArgs {
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A valid resource type. Valid Values: `AGENT_EVENTS` | `ATTACHMENTS` | `CALL_RECORDINGS` | `CHAT_TRANSCRIPTS` | `CONTACT_EVALUATIONS` | `CONTACT_TRACE_RECORDS` | `EMAIL_MESSAGES` | `MEDIA_STREAMS` | `REAL_TIME_CONTACT_ANALYSIS_CHAT_SEGMENTS` | `REAL_TIME_CONTACT_ANALYSIS_SEGMENTS` | `REAL_TIME_CONTACT_ANALYSIS_VOICE_SEGMENTS` | `SCHEDULED_REPORTS` | `SCREEN_RECORDINGS`.
     */
    resourceType: pulumi.Input<string>;
    /**
     * Specifies the storage configuration options for the Connect Instance. Documented below.
     */
    storageConfig: pulumi.Input<inputs.connect.InstanceStorageConfigStorageConfig>;
}
