import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Traffic mirror filter.\
 * Read [limits and considerations](https://docs.aws.amazon.com/vpc/latest/mirroring/traffic-mirroring-considerations.html) for traffic mirroring
 *
 * ## Example Usage
 *
 * To create a basic traffic mirror filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.ec2.TrafficMirrorFilter("foo", {
 *     description: "traffic mirror filter - example",
 *     networkServices: ["amazon-dns"],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import traffic mirror filter using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/trafficMirrorFilter:TrafficMirrorFilter foo tmf-0fbb93ddf38198f64
 * ```
 */
export declare class TrafficMirrorFilter extends pulumi.CustomResource {
    /**
     * Get an existing TrafficMirrorFilter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrafficMirrorFilterState, opts?: pulumi.CustomResourceOptions): TrafficMirrorFilter;
    /**
     * Returns true if the given object is an instance of TrafficMirrorFilter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrafficMirrorFilter;
    /**
     * The ARN of the traffic mirror filter.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the filter.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * List of amazon network services that should be mirrored. Valid values: `amazon-dns`.
     */
    readonly networkServices: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a TrafficMirrorFilter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TrafficMirrorFilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrafficMirrorFilter resources.
 */
export interface TrafficMirrorFilterState {
    /**
     * The ARN of the traffic mirror filter.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the filter.
     */
    description?: pulumi.Input<string>;
    /**
     * List of amazon network services that should be mirrored. Valid values: `amazon-dns`.
     */
    networkServices?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a TrafficMirrorFilter resource.
 */
export interface TrafficMirrorFilterArgs {
    /**
     * A description of the filter.
     */
    description?: pulumi.Input<string>;
    /**
     * List of amazon network services that should be mirrored. Valid values: `amazon-dns`.
     */
    networkServices?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
