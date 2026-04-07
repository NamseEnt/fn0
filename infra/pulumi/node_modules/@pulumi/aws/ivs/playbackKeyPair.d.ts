import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS IVS (Interactive Video) Playback Key Pair.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.ivs.PlaybackKeyPair("example", {publicKey: std.file({
 *     input: "./public-key.pem",
 * }).then(invoke => invoke.result)});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the IVS playback key pair.
 *
 * Using `pulumi import`, import IVS (Interactive Video) Playback Key Pair using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_ivs_playback_key_pair.example arn:aws:ivs:us-west-2:326937407773:playback-key/KDJRJNQhiQzA
 */
export declare class PlaybackKeyPair extends pulumi.CustomResource {
    /**
     * Get an existing PlaybackKeyPair resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PlaybackKeyPairState, opts?: pulumi.CustomResourceOptions): PlaybackKeyPair;
    /**
     * Returns true if the given object is an instance of PlaybackKeyPair.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PlaybackKeyPair;
    /**
     * ARN of the Playback Key Pair.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Key-pair identifier.
     */
    readonly fingerprint: pulumi.Output<string>;
    /**
     * Playback Key Pair name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Public portion of a customer-generated key pair. Must be an ECDSA public key in PEM format.
     *
     * The following arguments are optional:
     */
    readonly publicKey: pulumi.Output<string>;
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
     * Create a PlaybackKeyPair resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PlaybackKeyPairArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PlaybackKeyPair resources.
 */
export interface PlaybackKeyPairState {
    /**
     * ARN of the Playback Key Pair.
     */
    arn?: pulumi.Input<string>;
    /**
     * Key-pair identifier.
     */
    fingerprint?: pulumi.Input<string>;
    /**
     * Playback Key Pair name.
     */
    name?: pulumi.Input<string>;
    /**
     * Public portion of a customer-generated key pair. Must be an ECDSA public key in PEM format.
     *
     * The following arguments are optional:
     */
    publicKey?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a PlaybackKeyPair resource.
 */
export interface PlaybackKeyPairArgs {
    /**
     * Playback Key Pair name.
     */
    name?: pulumi.Input<string>;
    /**
     * Public portion of a customer-generated key pair. Must be an ECDSA public key in PEM format.
     *
     * The following arguments are optional:
     */
    publicKey: pulumi.Input<string>;
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
