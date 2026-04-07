import * as pulumi from "@pulumi/pulumi";
/**
 * Creates an AWS Elemental MediaPackage Version 2 Channel Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mediapackagev2.ChannelGroup("example", {
 *     name: "example",
 *     description: "channel group for example channels",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Elemental MediaPackage Version 2 Channel Group using the channel group's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:mediapackagev2/channelGroup:ChannelGroup example example
 * ```
 */
export declare class ChannelGroup extends pulumi.CustomResource {
    /**
     * Get an existing ChannelGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ChannelGroupState, opts?: pulumi.CustomResourceOptions): ChannelGroup;
    /**
     * Returns true if the given object is an instance of ChannelGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ChannelGroup;
    /**
     * The ARN of the channel
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the channel group
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The egress domain of the channel group
     */
    readonly egressDomain: pulumi.Output<string>;
    /**
     * A unique identifier naming the channel group
     */
    readonly name: pulumi.Output<string>;
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
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ChannelGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ChannelGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ChannelGroup resources.
 */
export interface ChannelGroupState {
    /**
     * The ARN of the channel
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the channel group
     */
    description?: pulumi.Input<string>;
    /**
     * The egress domain of the channel group
     */
    egressDomain?: pulumi.Input<string>;
    /**
     * A unique identifier naming the channel group
     */
    name?: pulumi.Input<string>;
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
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ChannelGroup resource.
 */
export interface ChannelGroupArgs {
    /**
     * A description of the channel group
     */
    description?: pulumi.Input<string>;
    /**
     * A unique identifier naming the channel group
     */
    name?: pulumi.Input<string>;
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
