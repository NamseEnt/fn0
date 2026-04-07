import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS MediaLive Multiplex.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 * });
 * const example = new aws.medialive.Multiplex("example", {
 *     name: "example-multiplex-changed",
 *     availabilityZones: [
 *         available.then(available => available.names?.[0]),
 *         available.then(available => available.names?.[1]),
 *     ],
 *     multiplexSettings: {
 *         transportStreamBitrate: 1000000,
 *         transportStreamId: 1,
 *         transportStreamReservedBitrate: 1,
 *         maximumVideoBufferDelayMilliseconds: 1000,
 *     },
 *     startMultiplex: true,
 *     tags: {
 *         tag1: "value1",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MediaLive Multiplex using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:medialive/multiplex:Multiplex example 12345678
 * ```
 */
export declare class Multiplex extends pulumi.CustomResource {
    /**
     * Get an existing Multiplex resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MultiplexState, opts?: pulumi.CustomResourceOptions): Multiplex;
    /**
     * Returns true if the given object is an instance of Multiplex.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Multiplex;
    /**
     * ARN of the Multiplex.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A list of availability zones. You must specify exactly two.
     */
    readonly availabilityZones: pulumi.Output<string[]>;
    /**
     * Multiplex settings. See Multiplex Settings for more details.
     */
    readonly multiplexSettings: pulumi.Output<outputs.medialive.MultiplexMultiplexSettings | undefined>;
    /**
     * name of Multiplex.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether to start the Multiplex. Defaults to `false`.
     */
    readonly startMultiplex: pulumi.Output<boolean | undefined>;
    /**
     * A map of tags to assign to the Multiplex. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Multiplex resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MultiplexArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Multiplex resources.
 */
export interface MultiplexState {
    /**
     * ARN of the Multiplex.
     */
    arn?: pulumi.Input<string>;
    /**
     * A list of availability zones. You must specify exactly two.
     */
    availabilityZones?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Multiplex settings. See Multiplex Settings for more details.
     */
    multiplexSettings?: pulumi.Input<inputs.medialive.MultiplexMultiplexSettings>;
    /**
     * name of Multiplex.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to start the Multiplex. Defaults to `false`.
     */
    startMultiplex?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the Multiplex. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Multiplex resource.
 */
export interface MultiplexArgs {
    /**
     * A list of availability zones. You must specify exactly two.
     */
    availabilityZones: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Multiplex settings. See Multiplex Settings for more details.
     */
    multiplexSettings?: pulumi.Input<inputs.medialive.MultiplexMultiplexSettings>;
    /**
     * name of Multiplex.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to start the Multiplex. Defaults to `false`.
     */
    startMultiplex?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the Multiplex. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
