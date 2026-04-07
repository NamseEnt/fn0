import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an SES receipt rule set resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.ses.ReceiptRuleSet("main", {ruleSetName: "primary-rules"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SES receipt rule sets using the rule set name. For example:
 *
 * ```sh
 * $ pulumi import aws:ses/receiptRuleSet:ReceiptRuleSet my_rule_set my_rule_set_name
 * ```
 */
export declare class ReceiptRuleSet extends pulumi.CustomResource {
    /**
     * Get an existing ReceiptRuleSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReceiptRuleSetState, opts?: pulumi.CustomResourceOptions): ReceiptRuleSet;
    /**
     * Returns true if the given object is an instance of ReceiptRuleSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReceiptRuleSet;
    /**
     * SES receipt rule set ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the rule set.
     */
    readonly ruleSetName: pulumi.Output<string>;
    /**
     * Create a ReceiptRuleSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReceiptRuleSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReceiptRuleSet resources.
 */
export interface ReceiptRuleSetState {
    /**
     * SES receipt rule set ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the rule set.
     */
    ruleSetName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ReceiptRuleSet resource.
 */
export interface ReceiptRuleSetArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the rule set.
     */
    ruleSetName: pulumi.Input<string>;
}
