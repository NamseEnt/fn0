import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon Managed Service for Prometheus (AMP) Rule Group Namespace
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const demo = new aws.amp.Workspace("demo", {});
 * const demoRuleGroupNamespace = new aws.amp.RuleGroupNamespace("demo", {
 *     name: "rules",
 *     workspaceId: demo.id,
 *     data: `groups:
 *   - name: test
 *     rules:
 *     - record: metric:recording_rule
 *       expr: avg(rate(container_cpu_usage_seconds_total[5m]))
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the prometheus rule group namespace using the arn. For example:
 *
 * ```sh
 * $ pulumi import aws:amp/ruleGroupNamespace:RuleGroupNamespace demo arn:aws:aps:us-west-2:123456789012:rulegroupsnamespace/IDstring/namespace_name
 * ```
 */
export declare class RuleGroupNamespace extends pulumi.CustomResource {
    /**
     * Get an existing RuleGroupNamespace resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RuleGroupNamespaceState, opts?: pulumi.CustomResourceOptions): RuleGroupNamespace;
    /**
     * Returns true if the given object is an instance of RuleGroupNamespace.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RuleGroupNamespace;
    /**
     * The ARN of the rule group namespace.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * the rule group namespace data that you want to be applied. See more [in AWS Docs](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-Ruler.html).
     */
    readonly data: pulumi.Output<string>;
    /**
     * The name of the rule group namespace.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ID of the prometheus workspace the rule group namespace should be linked to.
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a RuleGroupNamespace resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RuleGroupNamespaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RuleGroupNamespace resources.
 */
export interface RuleGroupNamespaceState {
    /**
     * The ARN of the rule group namespace.
     */
    arn?: pulumi.Input<string>;
    /**
     * the rule group namespace data that you want to be applied. See more [in AWS Docs](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-Ruler.html).
     */
    data?: pulumi.Input<string>;
    /**
     * The name of the rule group namespace.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ID of the prometheus workspace the rule group namespace should be linked to.
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RuleGroupNamespace resource.
 */
export interface RuleGroupNamespaceArgs {
    /**
     * the rule group namespace data that you want to be applied. See more [in AWS Docs](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-Ruler.html).
     */
    data: pulumi.Input<string>;
    /**
     * The name of the rule group namespace.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ID of the prometheus workspace the rule group namespace should be linked to.
     */
    workspaceId: pulumi.Input<string>;
}
