import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS MediaLive Channel.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.medialive.Channel("example", {
 *     name: "example-channel",
 *     channelClass: "STANDARD",
 *     roleArn: exampleAwsIamRole.arn,
 *     inputSpecification: {
 *         codec: "AVC",
 *         inputResolution: "HD",
 *         maximumBitrate: "MAX_20_MBPS",
 *     },
 *     inputAttachments: [{
 *         inputAttachmentName: "example-input",
 *         inputId: exampleAwsMedialiveInput.id,
 *     }],
 *     destinations: [{
 *         id: "destination",
 *         settings: [
 *             {
 *                 url: `s3://${main.id}/test1`,
 *             },
 *             {
 *                 url: `s3://${main2.id}/test2`,
 *             },
 *         ],
 *     }],
 *     encoderSettings: {
 *         timecodeConfig: {
 *             source: "EMBEDDED",
 *         },
 *         audioDescriptions: [{
 *             audioSelectorName: "example audio selector",
 *             name: "audio-selector",
 *         }],
 *         videoDescriptions: [{
 *             name: "example-video",
 *         }],
 *         outputGroups: [{
 *             outputGroupSettings: {
 *                 archiveGroupSettings: [{
 *                     destination: {
 *                         destinationRefId: "destination",
 *                     },
 *                 }],
 *             },
 *             outputs: [{
 *                 outputName: "example-name",
 *                 videoDescriptionName: "example-video",
 *                 audioDescriptionNames: ["audio-selector"],
 *                 outputSettings: {
 *                     archiveOutputSettings: {
 *                         nameModifier: "_1",
 *                         extension: "m2ts",
 *                         containerSettings: {
 *                             m2tsSettings: {
 *                                 audioBufferModel: "ATSC",
 *                                 bufferModel: "MULTIPLEX",
 *                                 rateMode: "CBR",
 *                             },
 *                         },
 *                     },
 *                 },
 *             }],
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MediaLive Channel using the `channel_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:medialive/channel:Channel example 1234567
 * ```
 */
export declare class Channel extends pulumi.CustomResource {
    /**
     * Get an existing Channel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ChannelState, opts?: pulumi.CustomResourceOptions): Channel;
    /**
     * Returns true if the given object is an instance of Channel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Channel;
    /**
     * ARN of the Channel.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specification of CDI inputs for this channel. See CDI Input Specification for more details.
     */
    readonly cdiInputSpecification: pulumi.Output<outputs.medialive.ChannelCdiInputSpecification | undefined>;
    /**
     * Concise argument description.
     */
    readonly channelClass: pulumi.Output<string>;
    /**
     * ID of the Channel.
     */
    readonly channelId: pulumi.Output<string>;
    /**
     * Destinations for channel. See Destinations for more details.
     */
    readonly destinations: pulumi.Output<outputs.medialive.ChannelDestination[]>;
    /**
     * Encoder settings. See Encoder Settings for more details.
     */
    readonly encoderSettings: pulumi.Output<outputs.medialive.ChannelEncoderSettings>;
    /**
     * Input attachments for the channel. See Input Attachments for more details.
     */
    readonly inputAttachments: pulumi.Output<outputs.medialive.ChannelInputAttachment[]>;
    /**
     * Specification of network and file inputs for the channel.
     */
    readonly inputSpecification: pulumi.Output<outputs.medialive.ChannelInputSpecification>;
    /**
     * The log level to write to Cloudwatch logs.
     */
    readonly logLevel: pulumi.Output<string>;
    /**
     * Maintenance settings for this channel. See Maintenance for more details.
     */
    readonly maintenance: pulumi.Output<outputs.medialive.ChannelMaintenance>;
    /**
     * Name of the Channel.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Concise argument description.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    /**
     * Whether to start/stop channel. Default: `false`
     */
    readonly startChannel: pulumi.Output<boolean | undefined>;
    /**
     * A map of tags to assign to the channel. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Settings for the VPC outputs. See VPC for more details.
     */
    readonly vpc: pulumi.Output<outputs.medialive.ChannelVpc | undefined>;
    /**
     * Create a Channel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ChannelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Channel resources.
 */
export interface ChannelState {
    /**
     * ARN of the Channel.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specification of CDI inputs for this channel. See CDI Input Specification for more details.
     */
    cdiInputSpecification?: pulumi.Input<inputs.medialive.ChannelCdiInputSpecification>;
    /**
     * Concise argument description.
     */
    channelClass?: pulumi.Input<string>;
    /**
     * ID of the Channel.
     */
    channelId?: pulumi.Input<string>;
    /**
     * Destinations for channel. See Destinations for more details.
     */
    destinations?: pulumi.Input<pulumi.Input<inputs.medialive.ChannelDestination>[]>;
    /**
     * Encoder settings. See Encoder Settings for more details.
     */
    encoderSettings?: pulumi.Input<inputs.medialive.ChannelEncoderSettings>;
    /**
     * Input attachments for the channel. See Input Attachments for more details.
     */
    inputAttachments?: pulumi.Input<pulumi.Input<inputs.medialive.ChannelInputAttachment>[]>;
    /**
     * Specification of network and file inputs for the channel.
     */
    inputSpecification?: pulumi.Input<inputs.medialive.ChannelInputSpecification>;
    /**
     * The log level to write to Cloudwatch logs.
     */
    logLevel?: pulumi.Input<string>;
    /**
     * Maintenance settings for this channel. See Maintenance for more details.
     */
    maintenance?: pulumi.Input<inputs.medialive.ChannelMaintenance>;
    /**
     * Name of the Channel.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Concise argument description.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Whether to start/stop channel. Default: `false`
     */
    startChannel?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the channel. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Settings for the VPC outputs. See VPC for more details.
     */
    vpc?: pulumi.Input<inputs.medialive.ChannelVpc>;
}
/**
 * The set of arguments for constructing a Channel resource.
 */
export interface ChannelArgs {
    /**
     * Specification of CDI inputs for this channel. See CDI Input Specification for more details.
     */
    cdiInputSpecification?: pulumi.Input<inputs.medialive.ChannelCdiInputSpecification>;
    /**
     * Concise argument description.
     */
    channelClass: pulumi.Input<string>;
    /**
     * Destinations for channel. See Destinations for more details.
     */
    destinations: pulumi.Input<pulumi.Input<inputs.medialive.ChannelDestination>[]>;
    /**
     * Encoder settings. See Encoder Settings for more details.
     */
    encoderSettings: pulumi.Input<inputs.medialive.ChannelEncoderSettings>;
    /**
     * Input attachments for the channel. See Input Attachments for more details.
     */
    inputAttachments: pulumi.Input<pulumi.Input<inputs.medialive.ChannelInputAttachment>[]>;
    /**
     * Specification of network and file inputs for the channel.
     */
    inputSpecification: pulumi.Input<inputs.medialive.ChannelInputSpecification>;
    /**
     * The log level to write to Cloudwatch logs.
     */
    logLevel?: pulumi.Input<string>;
    /**
     * Maintenance settings for this channel. See Maintenance for more details.
     */
    maintenance?: pulumi.Input<inputs.medialive.ChannelMaintenance>;
    /**
     * Name of the Channel.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Concise argument description.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Whether to start/stop channel. Default: `false`
     */
    startChannel?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the channel. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Settings for the VPC outputs. See VPC for more details.
     */
    vpc?: pulumi.Input<inputs.medialive.ChannelVpc>;
}
