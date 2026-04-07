import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Transcoder preset resource.
 *
 * > **Warning:** This resource is deprecated. Use [AWS Elemental MediaConvert](https://aws.amazon.com/blogs/media/migrating-workflows-from-amazon-elastic-transcoder-to-aws-elemental-mediaconvert/) instead. AWS will [discontinue support for Amazon Elastic Transcoder](https://aws.amazon.com/blogs/media/support-for-amazon-elastic-transcoder-ending-soon/), effective November 13, 2025.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const bar = new aws.elastictranscoder.Preset("bar", {
 *     container: "mp4",
 *     description: "Sample Preset",
 *     name: "sample_preset",
 *     audio: {
 *         audioPackingMode: "SingleTrack",
 *         bitRate: "96",
 *         channels: "2",
 *         codec: "AAC",
 *         sampleRate: "44100",
 *     },
 *     audioCodecOptions: {
 *         profile: "AAC-LC",
 *     },
 *     video: {
 *         bitRate: "1600",
 *         codec: "H.264",
 *         displayAspectRatio: "16:9",
 *         fixedGop: "false",
 *         frameRate: "auto",
 *         maxFrameRate: "60",
 *         keyframesMaxDist: "240",
 *         maxHeight: "auto",
 *         maxWidth: "auto",
 *         paddingPolicy: "Pad",
 *         sizingPolicy: "Fit",
 *     },
 *     videoCodecOptions: {
 *         Profile: "main",
 *         Level: "2.2",
 *         MaxReferenceFrames: "3",
 *         InterlacedMode: "Progressive",
 *         ColorSpaceConversionMode: "None",
 *     },
 *     videoWatermarks: [{
 *         id: "Test",
 *         maxWidth: "20%",
 *         maxHeight: "20%",
 *         sizingPolicy: "ShrinkToFit",
 *         horizontalAlign: "Right",
 *         horizontalOffset: "10px",
 *         verticalAlign: "Bottom",
 *         verticalOffset: "10px",
 *         opacity: "55.5",
 *         target: "Content",
 *     }],
 *     thumbnails: {
 *         format: "png",
 *         interval: "120",
 *         maxWidth: "auto",
 *         maxHeight: "auto",
 *         paddingPolicy: "Pad",
 *         sizingPolicy: "Fit",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Elastic Transcoder presets using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:elastictranscoder/preset:Preset basic_preset 1407981661351-cttk8b
 * ```
 */
export declare class Preset extends pulumi.CustomResource {
    /**
     * Get an existing Preset resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PresetState, opts?: pulumi.CustomResourceOptions): Preset;
    /**
     * Returns true if the given object is an instance of Preset.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Preset;
    /**
     * Amazon Resource Name (ARN) of the Elastic Transcoder Preset.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Audio parameters object (documented below).
     */
    readonly audio: pulumi.Output<outputs.elastictranscoder.PresetAudio | undefined>;
    /**
     * Codec options for the audio parameters (documented below)
     */
    readonly audioCodecOptions: pulumi.Output<outputs.elastictranscoder.PresetAudioCodecOptions>;
    /**
     * The container type for the output file. Valid values are `flac`, `flv`, `fmp4`, `gif`, `mp3`, `mp4`, `mpg`, `mxf`, `oga`, `ogg`, `ts`, and `webm`.
     */
    readonly container: pulumi.Output<string>;
    /**
     * A description of the preset (maximum 255 characters)
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the preset. (maximum 40 characters)
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Thumbnail parameters object (documented below)
     */
    readonly thumbnails: pulumi.Output<outputs.elastictranscoder.PresetThumbnails | undefined>;
    readonly type: pulumi.Output<string>;
    /**
     * Video parameters object (documented below)
     */
    readonly video: pulumi.Output<outputs.elastictranscoder.PresetVideo | undefined>;
    /**
     * Codec options for the video parameters
     *
     * See ["Create Preset"](http://docs.aws.amazon.com/elastictranscoder/latest/developerguide/create-preset.html) in the AWS docs for reference.
     */
    readonly videoCodecOptions: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Watermark parameters for the video parameters (documented below)
     */
    readonly videoWatermarks: pulumi.Output<outputs.elastictranscoder.PresetVideoWatermark[] | undefined>;
    /**
     * Create a Preset resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PresetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Preset resources.
 */
export interface PresetState {
    /**
     * Amazon Resource Name (ARN) of the Elastic Transcoder Preset.
     */
    arn?: pulumi.Input<string>;
    /**
     * Audio parameters object (documented below).
     */
    audio?: pulumi.Input<inputs.elastictranscoder.PresetAudio>;
    /**
     * Codec options for the audio parameters (documented below)
     */
    audioCodecOptions?: pulumi.Input<inputs.elastictranscoder.PresetAudioCodecOptions>;
    /**
     * The container type for the output file. Valid values are `flac`, `flv`, `fmp4`, `gif`, `mp3`, `mp4`, `mpg`, `mxf`, `oga`, `ogg`, `ts`, and `webm`.
     */
    container?: pulumi.Input<string>;
    /**
     * A description of the preset (maximum 255 characters)
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the preset. (maximum 40 characters)
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Thumbnail parameters object (documented below)
     */
    thumbnails?: pulumi.Input<inputs.elastictranscoder.PresetThumbnails>;
    type?: pulumi.Input<string>;
    /**
     * Video parameters object (documented below)
     */
    video?: pulumi.Input<inputs.elastictranscoder.PresetVideo>;
    /**
     * Codec options for the video parameters
     *
     * See ["Create Preset"](http://docs.aws.amazon.com/elastictranscoder/latest/developerguide/create-preset.html) in the AWS docs for reference.
     */
    videoCodecOptions?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Watermark parameters for the video parameters (documented below)
     */
    videoWatermarks?: pulumi.Input<pulumi.Input<inputs.elastictranscoder.PresetVideoWatermark>[]>;
}
/**
 * The set of arguments for constructing a Preset resource.
 */
export interface PresetArgs {
    /**
     * Audio parameters object (documented below).
     */
    audio?: pulumi.Input<inputs.elastictranscoder.PresetAudio>;
    /**
     * Codec options for the audio parameters (documented below)
     */
    audioCodecOptions?: pulumi.Input<inputs.elastictranscoder.PresetAudioCodecOptions>;
    /**
     * The container type for the output file. Valid values are `flac`, `flv`, `fmp4`, `gif`, `mp3`, `mp4`, `mpg`, `mxf`, `oga`, `ogg`, `ts`, and `webm`.
     */
    container: pulumi.Input<string>;
    /**
     * A description of the preset (maximum 255 characters)
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the preset. (maximum 40 characters)
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Thumbnail parameters object (documented below)
     */
    thumbnails?: pulumi.Input<inputs.elastictranscoder.PresetThumbnails>;
    type?: pulumi.Input<string>;
    /**
     * Video parameters object (documented below)
     */
    video?: pulumi.Input<inputs.elastictranscoder.PresetVideo>;
    /**
     * Codec options for the video parameters
     *
     * See ["Create Preset"](http://docs.aws.amazon.com/elastictranscoder/latest/developerguide/create-preset.html) in the AWS docs for reference.
     */
    videoCodecOptions?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Watermark parameters for the video parameters (documented below)
     */
    videoWatermarks?: pulumi.Input<pulumi.Input<inputs.elastictranscoder.PresetVideoWatermark>[]>;
}
