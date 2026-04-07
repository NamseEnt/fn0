import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS IVS (Interactive Video) Channel.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ivs.Channel("example", {name: "channel-1"});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the IVS channel.
 *
 * Using `pulumi import`, import IVS (Interactive Video) Channel using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_ivs_channel.example arn:aws:ivs:us-west-2:326937407773:channel/0Y1lcs4U7jk5
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
     * If `true`, channel is private (enabled for playback authorization).
     */
    readonly authorized: pulumi.Output<boolean>;
    /**
     * Channel ingest endpoint, part of the definition of an ingest server, used when setting up streaming software.
     */
    readonly ingestEndpoint: pulumi.Output<string>;
    /**
     * Channel latency mode. Valid values: `NORMAL`, `LOW`.
     */
    readonly latencyMode: pulumi.Output<string>;
    /**
     * Channel name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Channel playback URL.
     */
    readonly playbackUrl: pulumi.Output<string>;
    /**
     * Recording configuration ARN.
     */
    readonly recordingConfigurationArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Channel type, which determines the allowable resolution and bitrate. Valid values: `STANDARD`, `BASIC`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a Channel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ChannelArgs, opts?: pulumi.CustomResourceOptions);
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
     * If `true`, channel is private (enabled for playback authorization).
     */
    authorized?: pulumi.Input<boolean>;
    /**
     * Channel ingest endpoint, part of the definition of an ingest server, used when setting up streaming software.
     */
    ingestEndpoint?: pulumi.Input<string>;
    /**
     * Channel latency mode. Valid values: `NORMAL`, `LOW`.
     */
    latencyMode?: pulumi.Input<string>;
    /**
     * Channel name.
     */
    name?: pulumi.Input<string>;
    /**
     * Channel playback URL.
     */
    playbackUrl?: pulumi.Input<string>;
    /**
     * Recording configuration ARN.
     */
    recordingConfigurationArn?: pulumi.Input<string>;
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
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Channel type, which determines the allowable resolution and bitrate. Valid values: `STANDARD`, `BASIC`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Channel resource.
 */
export interface ChannelArgs {
    /**
     * If `true`, channel is private (enabled for playback authorization).
     */
    authorized?: pulumi.Input<boolean>;
    /**
     * Channel latency mode. Valid values: `NORMAL`, `LOW`.
     */
    latencyMode?: pulumi.Input<string>;
    /**
     * Channel name.
     */
    name?: pulumi.Input<string>;
    /**
     * Recording configuration ARN.
     */
    recordingConfigurationArn?: pulumi.Input<string>;
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
    /**
     * Channel type, which determines the allowable resolution and bitrate. Valid values: `STANDARD`, `BASIC`.
     */
    type?: pulumi.Input<string>;
}
