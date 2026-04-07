import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudWatch Evidently Segment resource.
 *
 * > **Warning:** This resource is deprecated. Use [AWS AppConfig feature flags](https://aws.amazon.com/blogs/mt/using-aws-appconfig-feature-flags/) instead.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.evidently.Segment("example", {
 *     name: "example",
 *     pattern: "{\"Price\":[{\"numeric\":[\">\",10,\"<=\",20]}]}",
 *     tags: {
 *         Key1: "example Segment",
 *     },
 * });
 * ```
 *
 * ### With JSON object in pattern
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.evidently.Segment("example", {
 *     name: "example",
 *     pattern: `  {
 *     \\"Price\\": [
 *       {
 *         \\"numeric\\": [\\">\\",10,\\"<=\\",20]
 *       }
 *     ]
 *   }
 * `,
 *     tags: {
 *         Key1: "example Segment",
 *     },
 * });
 * ```
 *
 * ### With Description
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.evidently.Segment("example", {
 *     name: "example",
 *     pattern: "{\"Price\":[{\"numeric\":[\">\",10,\"<=\",20]}]}",
 *     description: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Evidently Segment using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:evidently/segment:Segment example arn:aws:evidently:us-west-2:123456789012:segment/example
 * ```
 */
export declare class Segment extends pulumi.CustomResource {
    /**
     * Get an existing Segment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SegmentState, opts?: pulumi.CustomResourceOptions): Segment;
    /**
     * Returns true if the given object is an instance of Segment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Segment;
    /**
     * The ARN of the segment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date and time that the segment is created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Specifies the description of the segment.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The number of experiments that this segment is used in. This count includes all current experiments, not just those that are currently running.
     */
    readonly experimentCount: pulumi.Output<number>;
    /**
     * The date and time that this segment was most recently updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * The number of launches that this segment is used in. This count includes all current launches, not just those that are currently running.
     */
    readonly launchCount: pulumi.Output<number>;
    /**
     * A name for the segment.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The pattern to use for the segment. For more information about pattern syntax, see [Segment rule pattern syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Evidently-segments.html#CloudWatch-Evidently-segments-syntax.html).
     */
    readonly pattern: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the segment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Segment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SegmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Segment resources.
 */
export interface SegmentState {
    /**
     * The ARN of the segment.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date and time that the segment is created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Specifies the description of the segment.
     */
    description?: pulumi.Input<string>;
    /**
     * The number of experiments that this segment is used in. This count includes all current experiments, not just those that are currently running.
     */
    experimentCount?: pulumi.Input<number>;
    /**
     * The date and time that this segment was most recently updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * The number of launches that this segment is used in. This count includes all current launches, not just those that are currently running.
     */
    launchCount?: pulumi.Input<number>;
    /**
     * A name for the segment.
     */
    name?: pulumi.Input<string>;
    /**
     * The pattern to use for the segment. For more information about pattern syntax, see [Segment rule pattern syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Evidently-segments.html#CloudWatch-Evidently-segments-syntax.html).
     */
    pattern?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the segment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Segment resource.
 */
export interface SegmentArgs {
    /**
     * Specifies the description of the segment.
     */
    description?: pulumi.Input<string>;
    /**
     * A name for the segment.
     */
    name?: pulumi.Input<string>;
    /**
     * The pattern to use for the segment. For more information about pattern syntax, see [Segment rule pattern syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-Evidently-segments.html#CloudWatch-Evidently-segments-syntax.html).
     */
    pattern: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the segment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
