import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an WAF Regional Rule Resource for use with Application Load Balancer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ipset = new aws.wafregional.IpSet("ipset", {
 *     name: "tfIPSet",
 *     ipSetDescriptors: [{
 *         type: "IPV4",
 *         value: "192.0.7.0/24",
 *     }],
 * });
 * const wafrule = new aws.wafregional.Rule("wafrule", {
 *     name: "tfWAFRule",
 *     metricName: "tfWAFRule",
 *     predicates: [{
 *         type: "IPMatch",
 *         dataId: ipset.id,
 *         negated: false,
 *     }],
 * });
 * ```
 *
 * ## Nested Fields
 *
 * ### `predicate`
 *
 * See the [WAF Documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_Predicate.html) for more information.
 *
 * #### Arguments
 *
 * * `type` - (Required) The type of predicate in a rule. Valid values: `ByteMatch`, `GeoMatch`, `IPMatch`, `RegexMatch`, `SizeConstraint`, `SqlInjectionMatch`, or `XssMatch`
 * * `dataId` - (Required) The unique identifier of a predicate, such as the ID of a `ByteMatchSet` or `IPSet`.
 * * `negated` - (Required) Whether to use the settings or the negated settings that you specified in the objects.
 *
 * ## Import
 *
 * Using `pulumi import`, import WAF Regional Rule using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/rule:Rule wafrule a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
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
    /**
     * The ARN of the WAF Regional Rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name or description for the Amazon CloudWatch metric of this rule.
     */
    readonly metricName: pulumi.Output<string>;
    /**
     * The name or description of the rule.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The objects to include in a rule (documented below).
     */
    readonly predicates: pulumi.Output<outputs.wafregional.RulePredicate[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    /**
     * The ARN of the WAF Regional Rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name or description for the Amazon CloudWatch metric of this rule.
     */
    metricName?: pulumi.Input<string>;
    /**
     * The name or description of the rule.
     */
    name?: pulumi.Input<string>;
    /**
     * The objects to include in a rule (documented below).
     */
    predicates?: pulumi.Input<pulumi.Input<inputs.wafregional.RulePredicate>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Rule resource.
 */
export interface RuleArgs {
    /**
     * The name or description for the Amazon CloudWatch metric of this rule.
     */
    metricName: pulumi.Input<string>;
    /**
     * The name or description of the rule.
     */
    name?: pulumi.Input<string>;
    /**
     * The objects to include in a rule (documented below).
     */
    predicates?: pulumi.Input<pulumi.Input<inputs.wafregional.RulePredicate>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
