import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an SES receipt rule resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Add a header to the email and store it in S3
 * const store = new aws.ses.ReceiptRule("store", {
 *     name: "store",
 *     ruleSetName: "default-rule-set",
 *     recipients: ["karen@example.com"],
 *     enabled: true,
 *     scanEnabled: true,
 *     addHeaderActions: [{
 *         headerName: "Custom-Header",
 *         headerValue: "Added by SES",
 *         position: 1,
 *     }],
 *     s3Actions: [{
 *         bucketName: "emails",
 *         position: 2,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SES receipt rules using the ruleset name and rule name separated by `:`. For example:
 *
 * ```sh
 * $ pulumi import aws:ses/receiptRule:ReceiptRule my_rule my_rule_set:my_rule
 * ```
 */
export declare class ReceiptRule extends pulumi.CustomResource {
    /**
     * Get an existing ReceiptRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReceiptRuleState, opts?: pulumi.CustomResourceOptions): ReceiptRule;
    /**
     * Returns true if the given object is an instance of ReceiptRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReceiptRule;
    /**
     * A list of Add Header Action blocks. Documented below.
     */
    readonly addHeaderActions: pulumi.Output<outputs.ses.ReceiptRuleAddHeaderAction[] | undefined>;
    /**
     * The name of the rule to place this rule after
     */
    readonly after: pulumi.Output<string | undefined>;
    /**
     * The SES receipt rule ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A list of Bounce Action blocks. Documented below.
     */
    readonly bounceActions: pulumi.Output<outputs.ses.ReceiptRuleBounceAction[] | undefined>;
    /**
     * If true, the rule will be enabled
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * A list of Lambda Action blocks. Documented below.
     */
    readonly lambdaActions: pulumi.Output<outputs.ses.ReceiptRuleLambdaAction[] | undefined>;
    /**
     * The name of the rule
     */
    readonly name: pulumi.Output<string>;
    /**
     * A list of email addresses
     */
    readonly recipients: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the rule set
     */
    readonly ruleSetName: pulumi.Output<string>;
    /**
     * A list of S3 Action blocks. Documented below.
     */
    readonly s3Actions: pulumi.Output<outputs.ses.ReceiptRuleS3Action[] | undefined>;
    /**
     * If true, incoming emails will be scanned for spam and viruses
     */
    readonly scanEnabled: pulumi.Output<boolean | undefined>;
    /**
     * A list of SNS Action blocks. Documented below.
     */
    readonly snsActions: pulumi.Output<outputs.ses.ReceiptRuleSnsAction[] | undefined>;
    /**
     * A list of Stop Action blocks. Documented below.
     */
    readonly stopActions: pulumi.Output<outputs.ses.ReceiptRuleStopAction[] | undefined>;
    /**
     * `Require` or `Optional`
     */
    readonly tlsPolicy: pulumi.Output<string>;
    /**
     * A list of WorkMail Action blocks. Documented below.
     */
    readonly workmailActions: pulumi.Output<outputs.ses.ReceiptRuleWorkmailAction[] | undefined>;
    /**
     * Create a ReceiptRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReceiptRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReceiptRule resources.
 */
export interface ReceiptRuleState {
    /**
     * A list of Add Header Action blocks. Documented below.
     */
    addHeaderActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleAddHeaderAction>[]>;
    /**
     * The name of the rule to place this rule after
     */
    after?: pulumi.Input<string>;
    /**
     * The SES receipt rule ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * A list of Bounce Action blocks. Documented below.
     */
    bounceActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleBounceAction>[]>;
    /**
     * If true, the rule will be enabled
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * A list of Lambda Action blocks. Documented below.
     */
    lambdaActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleLambdaAction>[]>;
    /**
     * The name of the rule
     */
    name?: pulumi.Input<string>;
    /**
     * A list of email addresses
     */
    recipients?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the rule set
     */
    ruleSetName?: pulumi.Input<string>;
    /**
     * A list of S3 Action blocks. Documented below.
     */
    s3Actions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleS3Action>[]>;
    /**
     * If true, incoming emails will be scanned for spam and viruses
     */
    scanEnabled?: pulumi.Input<boolean>;
    /**
     * A list of SNS Action blocks. Documented below.
     */
    snsActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleSnsAction>[]>;
    /**
     * A list of Stop Action blocks. Documented below.
     */
    stopActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleStopAction>[]>;
    /**
     * `Require` or `Optional`
     */
    tlsPolicy?: pulumi.Input<string>;
    /**
     * A list of WorkMail Action blocks. Documented below.
     */
    workmailActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleWorkmailAction>[]>;
}
/**
 * The set of arguments for constructing a ReceiptRule resource.
 */
export interface ReceiptRuleArgs {
    /**
     * A list of Add Header Action blocks. Documented below.
     */
    addHeaderActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleAddHeaderAction>[]>;
    /**
     * The name of the rule to place this rule after
     */
    after?: pulumi.Input<string>;
    /**
     * A list of Bounce Action blocks. Documented below.
     */
    bounceActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleBounceAction>[]>;
    /**
     * If true, the rule will be enabled
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * A list of Lambda Action blocks. Documented below.
     */
    lambdaActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleLambdaAction>[]>;
    /**
     * The name of the rule
     */
    name?: pulumi.Input<string>;
    /**
     * A list of email addresses
     */
    recipients?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the rule set
     */
    ruleSetName: pulumi.Input<string>;
    /**
     * A list of S3 Action blocks. Documented below.
     */
    s3Actions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleS3Action>[]>;
    /**
     * If true, incoming emails will be scanned for spam and viruses
     */
    scanEnabled?: pulumi.Input<boolean>;
    /**
     * A list of SNS Action blocks. Documented below.
     */
    snsActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleSnsAction>[]>;
    /**
     * A list of Stop Action blocks. Documented below.
     */
    stopActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleStopAction>[]>;
    /**
     * `Require` or `Optional`
     */
    tlsPolicy?: pulumi.Input<string>;
    /**
     * A list of WorkMail Action blocks. Documented below.
     */
    workmailActions?: pulumi.Input<pulumi.Input<inputs.ses.ReceiptRuleWorkmailAction>[]>;
}
