import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS IVS (Interactive Video) Recording Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ivs.RecordingConfiguration("example", {
 *     name: "recording_configuration-1",
 *     destinationConfiguration: {
 *         s3: {
 *             bucketName: "ivs-stream-archive",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the IVS recording configuration.
 *
 * Using `pulumi import`, import IVS (Interactive Video) Recording Configuration using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_ivs_recording_configuration.example arn:aws:ivs:us-west-2:326937407773:recording-configuration/KAk1sHBl2L47
 */
export declare class RecordingConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing RecordingConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RecordingConfigurationState, opts?: pulumi.CustomResourceOptions): RecordingConfiguration;
    /**
     * Returns true if the given object is an instance of RecordingConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RecordingConfiguration;
    /**
     * ARN of the Recording Configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Object containing destination configuration for where recorded video will be stored.
     */
    readonly destinationConfiguration: pulumi.Output<outputs.ivs.RecordingConfigurationDestinationConfiguration>;
    /**
     * Recording Configuration name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * If a broadcast disconnects and then reconnects within the specified interval, the multiple streams will be considered a single broadcast and merged together.
     */
    readonly recordingReconnectWindowSeconds: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The current state of the Recording Configuration.
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
     * Object containing information to enable/disable the recording of thumbnails for a live session and modify the interval at which thumbnails are generated for the live session.
     */
    readonly thumbnailConfiguration: pulumi.Output<outputs.ivs.RecordingConfigurationThumbnailConfiguration>;
    /**
     * Create a RecordingConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RecordingConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RecordingConfiguration resources.
 */
export interface RecordingConfigurationState {
    /**
     * ARN of the Recording Configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Object containing destination configuration for where recorded video will be stored.
     */
    destinationConfiguration?: pulumi.Input<inputs.ivs.RecordingConfigurationDestinationConfiguration>;
    /**
     * Recording Configuration name.
     */
    name?: pulumi.Input<string>;
    /**
     * If a broadcast disconnects and then reconnects within the specified interval, the multiple streams will be considered a single broadcast and merged together.
     */
    recordingReconnectWindowSeconds?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The current state of the Recording Configuration.
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
    /**
     * Object containing information to enable/disable the recording of thumbnails for a live session and modify the interval at which thumbnails are generated for the live session.
     */
    thumbnailConfiguration?: pulumi.Input<inputs.ivs.RecordingConfigurationThumbnailConfiguration>;
}
/**
 * The set of arguments for constructing a RecordingConfiguration resource.
 */
export interface RecordingConfigurationArgs {
    /**
     * Object containing destination configuration for where recorded video will be stored.
     */
    destinationConfiguration: pulumi.Input<inputs.ivs.RecordingConfigurationDestinationConfiguration>;
    /**
     * Recording Configuration name.
     */
    name?: pulumi.Input<string>;
    /**
     * If a broadcast disconnects and then reconnects within the specified interval, the multiple streams will be considered a single broadcast and merged together.
     */
    recordingReconnectWindowSeconds?: pulumi.Input<number>;
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
     * Object containing information to enable/disable the recording of thumbnails for a live session and modify the interval at which thumbnails are generated for the live session.
     */
    thumbnailConfiguration?: pulumi.Input<inputs.ivs.RecordingConfigurationThumbnailConfiguration>;
}
