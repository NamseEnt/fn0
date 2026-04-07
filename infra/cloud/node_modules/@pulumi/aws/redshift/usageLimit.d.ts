import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a new Amazon Redshift Usage Limit.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.UsageLimit("example", {
 *     clusterIdentifier: exampleAwsRedshiftCluster.id,
 *     featureType: "concurrency-scaling",
 *     limitType: "time",
 *     amount: 60,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift usage limits using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/usageLimit:UsageLimit example example-id
 * ```
 */
export declare class UsageLimit extends pulumi.CustomResource {
    /**
     * Get an existing UsageLimit resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UsageLimitState, opts?: pulumi.CustomResourceOptions): UsageLimit;
    /**
     * Returns true if the given object is an instance of UsageLimit.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UsageLimit;
    /**
     * The limit amount. If time-based, this amount is in minutes. If data-based, this amount is in terabytes (TB). The value must be a positive number.
     */
    readonly amount: pulumi.Output<number>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Usage Limit.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The action that Amazon Redshift takes when the limit is reached. The default is `log`. Valid values are `log`, `emit-metric`, and `disable`.
     */
    readonly breachAction: pulumi.Output<string | undefined>;
    /**
     * The identifier of the cluster that you want to limit usage.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The Amazon Redshift feature that you want to limit. Valid values are `spectrum`, `concurrency-scaling`, and `cross-region-datasharing`.
     */
    readonly featureType: pulumi.Output<string>;
    /**
     * The type of limit. Depending on the feature type, this can be based on a time duration or data size. If FeatureType is `spectrum`, then LimitType must be `data-scanned`. If FeatureType is `concurrency-scaling`, then LimitType must be `time`. If FeatureType is `cross-region-datasharing`, then LimitType must be `data-scanned`. Valid values are `data-scanned`, and `time`.
     */
    readonly limitType: pulumi.Output<string>;
    /**
     * The time period that the amount applies to. A weekly period begins on Sunday. The default is `monthly`. Valid values are `daily`, `weekly`, and `monthly`.
     */
    readonly period: pulumi.Output<string | undefined>;
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
     * Create a UsageLimit resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UsageLimitArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UsageLimit resources.
 */
export interface UsageLimitState {
    /**
     * The limit amount. If time-based, this amount is in minutes. If data-based, this amount is in terabytes (TB). The value must be a positive number.
     */
    amount?: pulumi.Input<number>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Usage Limit.
     */
    arn?: pulumi.Input<string>;
    /**
     * The action that Amazon Redshift takes when the limit is reached. The default is `log`. Valid values are `log`, `emit-metric`, and `disable`.
     */
    breachAction?: pulumi.Input<string>;
    /**
     * The identifier of the cluster that you want to limit usage.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The Amazon Redshift feature that you want to limit. Valid values are `spectrum`, `concurrency-scaling`, and `cross-region-datasharing`.
     */
    featureType?: pulumi.Input<string>;
    /**
     * The type of limit. Depending on the feature type, this can be based on a time duration or data size. If FeatureType is `spectrum`, then LimitType must be `data-scanned`. If FeatureType is `concurrency-scaling`, then LimitType must be `time`. If FeatureType is `cross-region-datasharing`, then LimitType must be `data-scanned`. Valid values are `data-scanned`, and `time`.
     */
    limitType?: pulumi.Input<string>;
    /**
     * The time period that the amount applies to. A weekly period begins on Sunday. The default is `monthly`. Valid values are `daily`, `weekly`, and `monthly`.
     */
    period?: pulumi.Input<string>;
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
 * The set of arguments for constructing a UsageLimit resource.
 */
export interface UsageLimitArgs {
    /**
     * The limit amount. If time-based, this amount is in minutes. If data-based, this amount is in terabytes (TB). The value must be a positive number.
     */
    amount: pulumi.Input<number>;
    /**
     * The action that Amazon Redshift takes when the limit is reached. The default is `log`. Valid values are `log`, `emit-metric`, and `disable`.
     */
    breachAction?: pulumi.Input<string>;
    /**
     * The identifier of the cluster that you want to limit usage.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The Amazon Redshift feature that you want to limit. Valid values are `spectrum`, `concurrency-scaling`, and `cross-region-datasharing`.
     */
    featureType: pulumi.Input<string>;
    /**
     * The type of limit. Depending on the feature type, this can be based on a time duration or data size. If FeatureType is `spectrum`, then LimitType must be `data-scanned`. If FeatureType is `concurrency-scaling`, then LimitType must be `time`. If FeatureType is `cross-region-datasharing`, then LimitType must be `data-scanned`. Valid values are `data-scanned`, and `time`.
     */
    limitType: pulumi.Input<string>;
    /**
     * The time period that the amount applies to. A weekly period begins on Sunday. The default is `monthly`. Valid values are `daily`, `weekly`, and `monthly`.
     */
    period?: pulumi.Input<string>;
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
