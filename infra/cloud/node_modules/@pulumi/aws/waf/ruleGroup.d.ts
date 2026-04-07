import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a WAF Rule Group Resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.waf.Rule("example", {
 *     name: "example",
 *     metricName: "example",
 * });
 * const exampleRuleGroup = new aws.waf.RuleGroup("example", {
 *     name: "example",
 *     metricName: "example",
 *     activatedRules: [{
 *         action: {
 *             type: "COUNT",
 *         },
 *         priority: 50,
 *         ruleId: example.id,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WAF Rule Group using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:waf/ruleGroup:RuleGroup example a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
 * ```
 */
export declare class RuleGroup extends pulumi.CustomResource {
    /**
     * Get an existing RuleGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RuleGroupState, opts?: pulumi.CustomResourceOptions): RuleGroup;
    /**
     * Returns true if the given object is an instance of RuleGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RuleGroup;
    /**
     * A list of activated rules, see below
     */
    readonly activatedRules: pulumi.Output<outputs.waf.RuleGroupActivatedRule[] | undefined>;
    /**
     * The ARN of the WAF rule group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A friendly name for the metrics from the rule group
     */
    readonly metricName: pulumi.Output<string>;
    /**
     * Name of the rule group. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a RuleGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RuleGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RuleGroup resources.
 */
export interface RuleGroupState {
    /**
     * A list of activated rules, see below
     */
    activatedRules?: pulumi.Input<pulumi.Input<inputs.waf.RuleGroupActivatedRule>[]>;
    /**
     * The ARN of the WAF rule group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A friendly name for the metrics from the rule group
     */
    metricName?: pulumi.Input<string>;
    /**
     * Name of the rule group. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a RuleGroup resource.
 */
export interface RuleGroupArgs {
    /**
     * A list of activated rules, see below
     */
    activatedRules?: pulumi.Input<pulumi.Input<inputs.waf.RuleGroupActivatedRule>[]>;
    /**
     * A friendly name for the metrics from the rule group
     */
    metricName: pulumi.Input<string>;
    /**
     * Name of the rule group. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
