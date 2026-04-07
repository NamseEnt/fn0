import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS RBin Rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rbin.Rule("example", {
 *     description: "Example tag-level retention rule",
 *     resourceType: "EBS_SNAPSHOT",
 *     resourceTags: [{
 *         resourceTagKey: "tag_key",
 *         resourceTagValue: "tag_value",
 *     }],
 *     retentionPeriod: {
 *         retentionPeriodValue: 10,
 *         retentionPeriodUnit: "DAYS",
 *     },
 *     tags: {
 *         test_tag_key: "test_tag_value",
 *     },
 * });
 * ```
 *
 * ### Region-Level Retention Rule
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rbin.Rule("example", {
 *     description: "Example region-level retention rule with exclusion tags",
 *     resourceType: "EC2_IMAGE",
 *     excludeResourceTags: [{
 *         resourceTagKey: "tag_key",
 *         resourceTagValue: "tag_value",
 *     }],
 *     retentionPeriod: {
 *         retentionPeriodValue: 10,
 *         retentionPeriodUnit: "DAYS",
 *     },
 *     tags: {
 *         test_tag_key: "test_tag_value",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import RBin Rule using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:rbin/rule:Rule example examplerule
 * ```
 */
export declare class Rule extends pulumi.CustomResource {
    /**
     * Get an existing Rule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RuleState, opts?: pulumi.CustomResourceOptions): Rule;
    /**
     * Returns true if the given object is an instance of Rule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Rule;
    readonly arn: pulumi.Output<string>;
    /**
     * Retention rule description.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Exclusion tags to use to identify resources that are to be excluded, or ignored, by a Region-level retention rule. See `excludeResourceTags` below.
     */
    readonly excludeResourceTags: pulumi.Output<outputs.rbin.RuleExcludeResourceTag[] | undefined>;
    /**
     * Information about the retention rule lock configuration. See `lockConfiguration` below.
     */
    readonly lockConfiguration: pulumi.Output<outputs.rbin.RuleLockConfiguration | undefined>;
    /**
     * (Timestamp) Date and time at which the unlock delay is set to expire. Only returned for retention rules that have been unlocked and that are still within the unlock delay period.
     */
    readonly lockEndTime: pulumi.Output<string>;
    /**
     * (Optional) Lock state of the retention rules to list. Only retention rules with the specified lock state are returned. Valid values are `locked`, `pendingUnlock`, `unlocked`.
     */
    readonly lockState: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Resource tags to use to identify resources that are to be retained by a tag-level retention rule. See `resourceTags` below.
     */
    readonly resourceTags: pulumi.Output<outputs.rbin.RuleResourceTag[] | undefined>;
    /**
     * Resource type to be retained by the retention rule. Valid values are `EBS_SNAPSHOT` and `EC2_IMAGE`.
     */
    readonly resourceType: pulumi.Output<string>;
    /**
     * Information about the retention period for which the retention rule is to retain resources. See `retentionPeriod` below.
     *
     * The following arguments are optional:
     */
    readonly retentionPeriod: pulumi.Output<outputs.rbin.RuleRetentionPeriod>;
    /**
     * (String) State of the retention rule. Only retention rules that are in the `available` state retain resources. Valid values include `pending` and `available`.
     */
    readonly status: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Rule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Rule resources.
 */
export interface RuleState {
    arn?: pulumi.Input<string>;
    /**
     * Retention rule description.
     */
    description?: pulumi.Input<string>;
    /**
     * Exclusion tags to use to identify resources that are to be excluded, or ignored, by a Region-level retention rule. See `excludeResourceTags` below.
     */
    excludeResourceTags?: pulumi.Input<pulumi.Input<inputs.rbin.RuleExcludeResourceTag>[]>;
    /**
     * Information about the retention rule lock configuration. See `lockConfiguration` below.
     */
    lockConfiguration?: pulumi.Input<inputs.rbin.RuleLockConfiguration>;
    /**
     * (Timestamp) Date and time at which the unlock delay is set to expire. Only returned for retention rules that have been unlocked and that are still within the unlock delay period.
     */
    lockEndTime?: pulumi.Input<string>;
    /**
     * (Optional) Lock state of the retention rules to list. Only retention rules with the specified lock state are returned. Valid values are `locked`, `pendingUnlock`, `unlocked`.
     */
    lockState?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Resource tags to use to identify resources that are to be retained by a tag-level retention rule. See `resourceTags` below.
     */
    resourceTags?: pulumi.Input<pulumi.Input<inputs.rbin.RuleResourceTag>[]>;
    /**
     * Resource type to be retained by the retention rule. Valid values are `EBS_SNAPSHOT` and `EC2_IMAGE`.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Information about the retention period for which the retention rule is to retain resources. See `retentionPeriod` below.
     *
     * The following arguments are optional:
     */
    retentionPeriod?: pulumi.Input<inputs.rbin.RuleRetentionPeriod>;
    /**
     * (String) State of the retention rule. Only retention rules that are in the `available` state retain resources. Valid values include `pending` and `available`.
     */
    status?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Rule resource.
 */
export interface RuleArgs {
    /**
     * Retention rule description.
     */
    description?: pulumi.Input<string>;
    /**
     * Exclusion tags to use to identify resources that are to be excluded, or ignored, by a Region-level retention rule. See `excludeResourceTags` below.
     */
    excludeResourceTags?: pulumi.Input<pulumi.Input<inputs.rbin.RuleExcludeResourceTag>[]>;
    /**
     * Information about the retention rule lock configuration. See `lockConfiguration` below.
     */
    lockConfiguration?: pulumi.Input<inputs.rbin.RuleLockConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Resource tags to use to identify resources that are to be retained by a tag-level retention rule. See `resourceTags` below.
     */
    resourceTags?: pulumi.Input<pulumi.Input<inputs.rbin.RuleResourceTag>[]>;
    /**
     * Resource type to be retained by the retention rule. Valid values are `EBS_SNAPSHOT` and `EC2_IMAGE`.
     */
    resourceType: pulumi.Input<string>;
    /**
     * Information about the retention period for which the retention rule is to retain resources. See `retentionPeriod` below.
     *
     * The following arguments are optional:
     */
    retentionPeriod: pulumi.Input<inputs.rbin.RuleRetentionPeriod>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
