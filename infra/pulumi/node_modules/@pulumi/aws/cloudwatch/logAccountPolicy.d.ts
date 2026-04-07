import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudWatch Log Account Policy resource.
 *
 * ## Example Usage
 *
 * ### Account Data Protection Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const dataProtection = new aws.cloudwatch.LogAccountPolicy("data_protection", {
 *     policyName: "data-protection",
 *     policyType: "DATA_PROTECTION_POLICY",
 *     policyDocument: JSON.stringify({
 *         Name: "DataProtection",
 *         Version: "2021-06-01",
 *         Statement: [
 *             {
 *                 Sid: "Audit",
 *                 DataIdentifier: ["arn:aws:dataprotection::aws:data-identifier/EmailAddress"],
 *                 Operation: {
 *                     Audit: {
 *                         FindingsDestination: {},
 *                     },
 *                 },
 *             },
 *             {
 *                 Sid: "Redact",
 *                 DataIdentifier: ["arn:aws:dataprotection::aws:data-identifier/EmailAddress"],
 *                 Operation: {
 *                     Deidentify: {
 *                         MaskConfig: {},
 *                     },
 *                 },
 *             },
 *         ],
 *     }),
 * });
 * ```
 *
 * ### Subscription Filter Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const subscriptionFilter = new aws.cloudwatch.LogAccountPolicy("subscription_filter", {
 *     policyName: "subscription-filter",
 *     policyType: "SUBSCRIPTION_FILTER_POLICY",
 *     policyDocument: JSON.stringify({
 *         DestinationArn: test.arn,
 *         FilterPattern: "test",
 *     }),
 *     selectionCriteria: "LogGroupName NOT IN [\"excluded_log_group_name\"]",
 * });
 * ```
 *
 * ### Field Index Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const fieldIndex = new aws.cloudwatch.LogAccountPolicy("field_index", {
 *     policyName: "field-index",
 *     policyType: "FIELD_INDEX_POLICY",
 *     policyDocument: JSON.stringify({
 *         Fields: [
 *             "field1",
 *             "field2",
 *         ],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import this resource using the `policy_name` and `policy_type` separated by `:`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logAccountPolicy:LogAccountPolicy example "my-account-policy:SUBSCRIPTION_FILTER_POLICY"
 * ```
 */
export declare class LogAccountPolicy extends pulumi.CustomResource {
    /**
     * Get an existing LogAccountPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogAccountPolicyState, opts?: pulumi.CustomResourceOptions): LogAccountPolicy;
    /**
     * Returns true if the given object is an instance of LogAccountPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogAccountPolicy;
    /**
     * Text of the account policy. Refer to the [AWS docs](https://docs.aws.amazon.com/cli/latest/reference/logs/put-account-policy.html) for more information.
     */
    readonly policyDocument: pulumi.Output<string>;
    /**
     * Name of the account policy.
     */
    readonly policyName: pulumi.Output<string>;
    /**
     * Type of account policy. One of `DATA_PROTECTION_POLICY`, `SUBSCRIPTION_FILTER_POLICY`, `FIELD_INDEX_POLICY` or `TRANSFORMER_POLICY`. You can have one account policy per type in an account.
     */
    readonly policyType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Currently defaults to and only accepts the value: `ALL`.
     */
    readonly scope: pulumi.Output<string | undefined>;
    /**
     * Criteria for applying a subscription filter policy to a selection of log groups. The only allowable criteria selector is `LogGroupName NOT IN []`.
     */
    readonly selectionCriteria: pulumi.Output<string | undefined>;
    /**
     * Create a LogAccountPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogAccountPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogAccountPolicy resources.
 */
export interface LogAccountPolicyState {
    /**
     * Text of the account policy. Refer to the [AWS docs](https://docs.aws.amazon.com/cli/latest/reference/logs/put-account-policy.html) for more information.
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * Name of the account policy.
     */
    policyName?: pulumi.Input<string>;
    /**
     * Type of account policy. One of `DATA_PROTECTION_POLICY`, `SUBSCRIPTION_FILTER_POLICY`, `FIELD_INDEX_POLICY` or `TRANSFORMER_POLICY`. You can have one account policy per type in an account.
     */
    policyType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Currently defaults to and only accepts the value: `ALL`.
     */
    scope?: pulumi.Input<string>;
    /**
     * Criteria for applying a subscription filter policy to a selection of log groups. The only allowable criteria selector is `LogGroupName NOT IN []`.
     */
    selectionCriteria?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogAccountPolicy resource.
 */
export interface LogAccountPolicyArgs {
    /**
     * Text of the account policy. Refer to the [AWS docs](https://docs.aws.amazon.com/cli/latest/reference/logs/put-account-policy.html) for more information.
     */
    policyDocument: pulumi.Input<string>;
    /**
     * Name of the account policy.
     */
    policyName: pulumi.Input<string>;
    /**
     * Type of account policy. One of `DATA_PROTECTION_POLICY`, `SUBSCRIPTION_FILTER_POLICY`, `FIELD_INDEX_POLICY` or `TRANSFORMER_POLICY`. You can have one account policy per type in an account.
     */
    policyType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Currently defaults to and only accepts the value: `ALL`.
     */
    scope?: pulumi.Input<string>;
    /**
     * Criteria for applying a subscription filter policy to a selection of log groups. The only allowable criteria selector is `LogGroupName NOT IN []`.
     */
    selectionCriteria?: pulumi.Input<string>;
}
