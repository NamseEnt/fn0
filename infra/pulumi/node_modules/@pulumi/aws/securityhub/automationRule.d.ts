import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Security Hub Automation Rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.AutomationRule("example", {
 *     description: "Elevate finding severity to CRITICAL when specific resources such as an S3 bucket is at risk",
 *     ruleName: "Elevate severity of findings that relate to important resources",
 *     ruleOrder: 1,
 *     actions: [{
 *         findingFieldsUpdate: {
 *             severity: {
 *                 label: "CRITICAL",
 *                 product: 0,
 *             },
 *             note: {
 *                 text: "This is a critical resource. Please review ASAP.",
 *                 updatedBy: "sechub-automation",
 *             },
 *             types: ["Software and Configuration Checks/Industry and Regulatory Standards"],
 *             userDefinedFields: {
 *                 key: "value",
 *             },
 *         },
 *         type: "FINDING_FIELDS_UPDATE",
 *     }],
 *     criteria: {
 *         resourceIds: [{
 *             comparison: "EQUALS",
 *             value: "arn:aws:s3:::examplebucket/*",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Security Hub automation rule.
 *
 * Using `pulumi import`, import Security Hub automation rule using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_securityhub_automation_rule.example arn:aws:securityhub:us-west-2:123456789012:automation-rule/473eddde-f5c4-4ae5-85c7-e922f271fffc
 */
export declare class AutomationRule extends pulumi.CustomResource {
    /**
     * Get an existing AutomationRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AutomationRuleState, opts?: pulumi.CustomResourceOptions): AutomationRule;
    /**
     * Returns true if the given object is an instance of AutomationRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AutomationRule;
    /**
     * A block that specifies one or more actions to update finding fields if a finding matches the conditions specified in `Criteria`. Documented below.
     */
    readonly actions: pulumi.Output<outputs.securityhub.AutomationRuleAction[] | undefined>;
    /**
     * The ARN of the Security Hub automation rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A block that specifies a set of ASFF finding field attributes and corresponding expected values that Security Hub uses to filter findings. Documented below.
     */
    readonly criteria: pulumi.Output<outputs.securityhub.AutomationRuleCriteria | undefined>;
    /**
     * The description of the rule.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Specifies whether a rule is the last to be applied with respect to a finding that matches the rule criteria. Defaults to `false`.
     */
    readonly isTerminal: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the rule.
     */
    readonly ruleName: pulumi.Output<string>;
    /**
     * An integer ranging from 1 to 1000 that represents the order in which the rule action is applied to findings. Security Hub applies rules with lower values for this parameter first.
     */
    readonly ruleOrder: pulumi.Output<number>;
    /**
     * Whether the rule is active after it is created.
     */
    readonly ruleStatus: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a AutomationRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AutomationRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AutomationRule resources.
 */
export interface AutomationRuleState {
    /**
     * A block that specifies one or more actions to update finding fields if a finding matches the conditions specified in `Criteria`. Documented below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.securityhub.AutomationRuleAction>[]>;
    /**
     * The ARN of the Security Hub automation rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * A block that specifies a set of ASFF finding field attributes and corresponding expected values that Security Hub uses to filter findings. Documented below.
     */
    criteria?: pulumi.Input<inputs.securityhub.AutomationRuleCriteria>;
    /**
     * The description of the rule.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies whether a rule is the last to be applied with respect to a finding that matches the rule criteria. Defaults to `false`.
     */
    isTerminal?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the rule.
     */
    ruleName?: pulumi.Input<string>;
    /**
     * An integer ranging from 1 to 1000 that represents the order in which the rule action is applied to findings. Security Hub applies rules with lower values for this parameter first.
     */
    ruleOrder?: pulumi.Input<number>;
    /**
     * Whether the rule is active after it is created.
     */
    ruleStatus?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a AutomationRule resource.
 */
export interface AutomationRuleArgs {
    /**
     * A block that specifies one or more actions to update finding fields if a finding matches the conditions specified in `Criteria`. Documented below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.securityhub.AutomationRuleAction>[]>;
    /**
     * A block that specifies a set of ASFF finding field attributes and corresponding expected values that Security Hub uses to filter findings. Documented below.
     */
    criteria?: pulumi.Input<inputs.securityhub.AutomationRuleCriteria>;
    /**
     * The description of the rule.
     */
    description: pulumi.Input<string>;
    /**
     * Specifies whether a rule is the last to be applied with respect to a finding that matches the rule criteria. Defaults to `false`.
     */
    isTerminal?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the rule.
     */
    ruleName: pulumi.Input<string>;
    /**
     * An integer ranging from 1 to 1000 that represents the order in which the rule action is applied to findings. Security Hub applies rules with lower values for this parameter first.
     */
    ruleOrder: pulumi.Input<number>;
    /**
     * Whether the rule is active after it is created.
     */
    ruleStatus?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
