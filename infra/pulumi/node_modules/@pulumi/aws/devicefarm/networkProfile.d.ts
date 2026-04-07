import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage AWS Device Farm Network Profiles.
 * ∂
 * > **NOTE:** AWS currently has limited regional support for Device Farm (e.g., `us-west-2`). See [AWS Device Farm endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/devicefarm.html) for information on supported regions.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devicefarm.Project("example", {name: "example"});
 * const exampleNetworkProfile = new aws.devicefarm.NetworkProfile("example", {
 *     name: "example",
 *     projectArn: example.arn,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Device Farm network profile.
 *
 * Using `pulumi import`, import DeviceFarm Network Profiles using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_devicefarm_network_profile.example arn:aws:devicefarm:us-west-2:123456789012:networkprofile:4fa784c7-ccb4-4dbf-ba4f-02198320daa1
 */
export declare class NetworkProfile extends pulumi.CustomResource {
    /**
     * Get an existing NetworkProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkProfileState, opts?: pulumi.CustomResourceOptions): NetworkProfile;
    /**
     * Returns true if the given object is an instance of NetworkProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkProfile;
    /**
     * The Amazon Resource Name of this network profile.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the network profile.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The data throughput rate in bits per second, as an integer from `0` to `104857600`. Default value is `104857600`.
     */
    readonly downlinkBandwidthBits: pulumi.Output<number | undefined>;
    /**
     * Delay time for all packets to destination in milliseconds as an integer from `0` to `2000`.
     */
    readonly downlinkDelayMs: pulumi.Output<number | undefined>;
    /**
     * Time variation in the delay of received packets in milliseconds as an integer from `0` to `2000`.
     */
    readonly downlinkJitterMs: pulumi.Output<number | undefined>;
    /**
     * Proportion of received packets that fail to arrive from `0` to `100` percent.
     */
    readonly downlinkLossPercent: pulumi.Output<number | undefined>;
    /**
     * The name for the network profile.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ARN of the project for the network profile.
     */
    readonly projectArn: pulumi.Output<string>;
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
     * The type of network profile to create. Valid values are listed are `PRIVATE` and `CURATED`.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * The data throughput rate in bits per second, as an integer from `0` to `104857600`. Default value is `104857600`.
     */
    readonly uplinkBandwidthBits: pulumi.Output<number | undefined>;
    /**
     * Delay time for all packets to destination in milliseconds as an integer from `0` to `2000`.
     */
    readonly uplinkDelayMs: pulumi.Output<number | undefined>;
    /**
     * Time variation in the delay of received packets in milliseconds as an integer from `0` to `2000`.
     */
    readonly uplinkJitterMs: pulumi.Output<number | undefined>;
    /**
     * Proportion of received packets that fail to arrive from `0` to `100` percent.
     */
    readonly uplinkLossPercent: pulumi.Output<number | undefined>;
    /**
     * Create a NetworkProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkProfile resources.
 */
export interface NetworkProfileState {
    /**
     * The Amazon Resource Name of this network profile.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the network profile.
     */
    description?: pulumi.Input<string>;
    /**
     * The data throughput rate in bits per second, as an integer from `0` to `104857600`. Default value is `104857600`.
     */
    downlinkBandwidthBits?: pulumi.Input<number>;
    /**
     * Delay time for all packets to destination in milliseconds as an integer from `0` to `2000`.
     */
    downlinkDelayMs?: pulumi.Input<number>;
    /**
     * Time variation in the delay of received packets in milliseconds as an integer from `0` to `2000`.
     */
    downlinkJitterMs?: pulumi.Input<number>;
    /**
     * Proportion of received packets that fail to arrive from `0` to `100` percent.
     */
    downlinkLossPercent?: pulumi.Input<number>;
    /**
     * The name for the network profile.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the project for the network profile.
     */
    projectArn?: pulumi.Input<string>;
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
    /**
     * The type of network profile to create. Valid values are listed are `PRIVATE` and `CURATED`.
     */
    type?: pulumi.Input<string>;
    /**
     * The data throughput rate in bits per second, as an integer from `0` to `104857600`. Default value is `104857600`.
     */
    uplinkBandwidthBits?: pulumi.Input<number>;
    /**
     * Delay time for all packets to destination in milliseconds as an integer from `0` to `2000`.
     */
    uplinkDelayMs?: pulumi.Input<number>;
    /**
     * Time variation in the delay of received packets in milliseconds as an integer from `0` to `2000`.
     */
    uplinkJitterMs?: pulumi.Input<number>;
    /**
     * Proportion of received packets that fail to arrive from `0` to `100` percent.
     */
    uplinkLossPercent?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a NetworkProfile resource.
 */
export interface NetworkProfileArgs {
    /**
     * The description of the network profile.
     */
    description?: pulumi.Input<string>;
    /**
     * The data throughput rate in bits per second, as an integer from `0` to `104857600`. Default value is `104857600`.
     */
    downlinkBandwidthBits?: pulumi.Input<number>;
    /**
     * Delay time for all packets to destination in milliseconds as an integer from `0` to `2000`.
     */
    downlinkDelayMs?: pulumi.Input<number>;
    /**
     * Time variation in the delay of received packets in milliseconds as an integer from `0` to `2000`.
     */
    downlinkJitterMs?: pulumi.Input<number>;
    /**
     * Proportion of received packets that fail to arrive from `0` to `100` percent.
     */
    downlinkLossPercent?: pulumi.Input<number>;
    /**
     * The name for the network profile.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the project for the network profile.
     */
    projectArn: pulumi.Input<string>;
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
     * The type of network profile to create. Valid values are listed are `PRIVATE` and `CURATED`.
     */
    type?: pulumi.Input<string>;
    /**
     * The data throughput rate in bits per second, as an integer from `0` to `104857600`. Default value is `104857600`.
     */
    uplinkBandwidthBits?: pulumi.Input<number>;
    /**
     * Delay time for all packets to destination in milliseconds as an integer from `0` to `2000`.
     */
    uplinkDelayMs?: pulumi.Input<number>;
    /**
     * Time variation in the delay of received packets in milliseconds as an integer from `0` to `2000`.
     */
    uplinkJitterMs?: pulumi.Input<number>;
    /**
     * Proportion of received packets that fail to arrive from `0` to `100` percent.
     */
    uplinkLossPercent?: pulumi.Input<number>;
}
