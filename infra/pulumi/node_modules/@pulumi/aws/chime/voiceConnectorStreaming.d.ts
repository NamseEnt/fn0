import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Adds a streaming configuration for the specified Amazon Chime Voice Connector. The streaming configuration specifies whether media streaming is enabled for sending to Amazon Kinesis.
 * It also sets the retention period, in hours, for the Amazon Kinesis data.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.chime.VoiceConnector("default", {
 *     name: "vc-name-test",
 *     requireEncryption: true,
 * });
 * const defaultVoiceConnectorStreaming = new aws.chime.VoiceConnectorStreaming("default", {
 *     disabled: false,
 *     voiceConnectorId: _default.id,
 *     dataRetention: 7,
 *     streamingNotificationTargets: ["SQS"],
 * });
 * ```
 *
 * ### Example Usage With Media Insights
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.chime.VoiceConnector("default", {
 *     name: "vc-name-test",
 *     requireEncryption: true,
 * });
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["mediapipelines.chime.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "ExampleResourceAccessRole",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleStream = new aws.kinesis.Stream("example", {
 *     name: "ExampleStream",
 *     shardCount: 2,
 * });
 * const example = new aws.chimesdkmediapipelines.MediaInsightsPipelineConfiguration("example", {
 *     name: "ExampleConfig",
 *     resourceAccessRoleArn: exampleRole.arn,
 *     elements: [
 *         {
 *             type: "AmazonTranscribeCallAnalyticsProcessor",
 *             amazonTranscribeCallAnalyticsProcessorConfiguration: {
 *                 languageCode: "en-US",
 *             },
 *         },
 *         {
 *             type: "KinesisDataStreamSink",
 *             kinesisDataStreamSinkConfiguration: {
 *                 insightsTarget: exampleStream.arn,
 *             },
 *         },
 *     ],
 * });
 * const defaultVoiceConnectorStreaming = new aws.chime.VoiceConnectorStreaming("default", {
 *     disabled: false,
 *     voiceConnectorId: _default.id,
 *     dataRetention: 7,
 *     streamingNotificationTargets: ["SQS"],
 *     mediaInsightsConfiguration: {
 *         disabled: false,
 *         configurationArn: example.arn,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Chime Voice Connector Streaming using the `voice_connector_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/voiceConnectorStreaming:VoiceConnectorStreaming default abcdef1ghij2klmno3pqr4
 * ```
 */
export declare class VoiceConnectorStreaming extends pulumi.CustomResource {
    /**
     * Get an existing VoiceConnectorStreaming resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VoiceConnectorStreamingState, opts?: pulumi.CustomResourceOptions): VoiceConnectorStreaming;
    /**
     * Returns true if the given object is an instance of VoiceConnectorStreaming.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VoiceConnectorStreaming;
    /**
     * The retention period, in hours, for the Amazon Kinesis data.
     */
    readonly dataRetention: pulumi.Output<number>;
    /**
     * When true, media streaming to Amazon Kinesis is turned off. Default: `false`
     */
    readonly disabled: pulumi.Output<boolean | undefined>;
    /**
     * The media insights configuration. See `mediaInsightsConfiguration`.
     */
    readonly mediaInsightsConfiguration: pulumi.Output<outputs.chime.VoiceConnectorStreamingMediaInsightsConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The streaming notification targets. Valid Values: `EventBridge | SNS | SQS`
     */
    readonly streamingNotificationTargets: pulumi.Output<string[] | undefined>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    readonly voiceConnectorId: pulumi.Output<string>;
    /**
     * Create a VoiceConnectorStreaming resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VoiceConnectorStreamingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VoiceConnectorStreaming resources.
 */
export interface VoiceConnectorStreamingState {
    /**
     * The retention period, in hours, for the Amazon Kinesis data.
     */
    dataRetention?: pulumi.Input<number>;
    /**
     * When true, media streaming to Amazon Kinesis is turned off. Default: `false`
     */
    disabled?: pulumi.Input<boolean>;
    /**
     * The media insights configuration. See `mediaInsightsConfiguration`.
     */
    mediaInsightsConfiguration?: pulumi.Input<inputs.chime.VoiceConnectorStreamingMediaInsightsConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The streaming notification targets. Valid Values: `EventBridge | SNS | SQS`
     */
    streamingNotificationTargets?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    voiceConnectorId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VoiceConnectorStreaming resource.
 */
export interface VoiceConnectorStreamingArgs {
    /**
     * The retention period, in hours, for the Amazon Kinesis data.
     */
    dataRetention: pulumi.Input<number>;
    /**
     * When true, media streaming to Amazon Kinesis is turned off. Default: `false`
     */
    disabled?: pulumi.Input<boolean>;
    /**
     * The media insights configuration. See `mediaInsightsConfiguration`.
     */
    mediaInsightsConfiguration?: pulumi.Input<inputs.chime.VoiceConnectorStreamingMediaInsightsConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The streaming notification targets. Valid Values: `EventBridge | SNS | SQS`
     */
    streamingNotificationTargets?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    voiceConnectorId: pulumi.Input<string>;
}
