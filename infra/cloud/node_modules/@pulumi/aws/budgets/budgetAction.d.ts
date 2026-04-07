import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a budget action resource. Budget actions are cost savings controls that run either automatically on your behalf or by using a workflow approval process.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["ec2:Describe*"],
 *         resources: ["*"],
 *     }],
 * });
 * const examplePolicy = new aws.iam.Policy("example", {
 *     name: "example",
 *     description: "My example policy",
 *     policy: example.then(example => example.json),
 * });
 * const current = aws.getPartition({});
 * const assumeRole = current.then(current => aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: [`budgets.${current.dnsSuffix}`],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * }));
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleBudget = new aws.budgets.Budget("example", {
 *     name: "example",
 *     budgetType: "USAGE",
 *     limitAmount: "10.0",
 *     limitUnit: "dollars",
 *     timePeriodStart: "2006-01-02_15:04",
 *     timeUnit: "MONTHLY",
 * });
 * const exampleBudgetAction = new aws.budgets.BudgetAction("example", {
 *     budgetName: exampleBudget.name,
 *     actionType: "APPLY_IAM_POLICY",
 *     approvalModel: "AUTOMATIC",
 *     notificationType: "ACTUAL",
 *     executionRoleArn: exampleRole.arn,
 *     actionThreshold: {
 *         actionThresholdType: "ABSOLUTE_VALUE",
 *         actionThresholdValue: 100,
 *     },
 *     definition: {
 *         iamActionDefinition: {
 *             policyArn: examplePolicy.arn,
 *             roles: [exampleRole.name],
 *         },
 *     },
 *     subscribers: [{
 *         address: "example@example.example",
 *         subscriptionType: "EMAIL",
 *     }],
 *     tags: {
 *         Tag1: "Value1",
 *         Tag2: "Value2",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import budget actions using `AccountID:ActionID:BudgetName`. For example:
 *
 * ```sh
 * $ pulumi import aws:budgets/budgetAction:BudgetAction myBudget 123456789012:some-id:myBudget
 * ```
 */
export declare class BudgetAction extends pulumi.CustomResource {
    /**
     * Get an existing BudgetAction resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BudgetActionState, opts?: pulumi.CustomResourceOptions): BudgetAction;
    /**
     * Returns true if the given object is an instance of BudgetAction.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BudgetAction;
    /**
     * The ID of the target account for budget. Will use current user's accountId by default if omitted.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The id of the budget action.
     */
    readonly actionId: pulumi.Output<string>;
    /**
     * The trigger threshold of the action. See Action Threshold.
     */
    readonly actionThreshold: pulumi.Output<outputs.budgets.BudgetActionActionThreshold>;
    /**
     * The type of action. This defines the type of tasks that can be carried out by this action. This field also determines the format for definition. Valid values are `APPLY_IAM_POLICY`, `APPLY_SCP_POLICY`, and `RUN_SSM_DOCUMENTS`.
     */
    readonly actionType: pulumi.Output<string>;
    /**
     * This specifies if the action needs manual or automatic approval. Valid values are `AUTOMATIC` and `MANUAL`.
     */
    readonly approvalModel: pulumi.Output<string>;
    /**
     * The ARN of the budget action.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of a budget.
     */
    readonly budgetName: pulumi.Output<string>;
    /**
     * Specifies all of the type-specific parameters. See Definition.
     */
    readonly definition: pulumi.Output<outputs.budgets.BudgetActionDefinition>;
    /**
     * The role passed for action execution and reversion. Roles and actions must be in the same account.
     */
    readonly executionRoleArn: pulumi.Output<string>;
    /**
     * The type of a notification. Valid values are `ACTUAL` or `FORECASTED`.
     */
    readonly notificationType: pulumi.Output<string>;
    /**
     * The status of the budget action.
     */
    readonly status: pulumi.Output<string>;
    /**
     * A list of subscribers. See Subscriber.
     */
    readonly subscribers: pulumi.Output<outputs.budgets.BudgetActionSubscriber[]>;
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
     * Create a BudgetAction resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BudgetActionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BudgetAction resources.
 */
export interface BudgetActionState {
    /**
     * The ID of the target account for budget. Will use current user's accountId by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The id of the budget action.
     */
    actionId?: pulumi.Input<string>;
    /**
     * The trigger threshold of the action. See Action Threshold.
     */
    actionThreshold?: pulumi.Input<inputs.budgets.BudgetActionActionThreshold>;
    /**
     * The type of action. This defines the type of tasks that can be carried out by this action. This field also determines the format for definition. Valid values are `APPLY_IAM_POLICY`, `APPLY_SCP_POLICY`, and `RUN_SSM_DOCUMENTS`.
     */
    actionType?: pulumi.Input<string>;
    /**
     * This specifies if the action needs manual or automatic approval. Valid values are `AUTOMATIC` and `MANUAL`.
     */
    approvalModel?: pulumi.Input<string>;
    /**
     * The ARN of the budget action.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of a budget.
     */
    budgetName?: pulumi.Input<string>;
    /**
     * Specifies all of the type-specific parameters. See Definition.
     */
    definition?: pulumi.Input<inputs.budgets.BudgetActionDefinition>;
    /**
     * The role passed for action execution and reversion. Roles and actions must be in the same account.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * The type of a notification. Valid values are `ACTUAL` or `FORECASTED`.
     */
    notificationType?: pulumi.Input<string>;
    /**
     * The status of the budget action.
     */
    status?: pulumi.Input<string>;
    /**
     * A list of subscribers. See Subscriber.
     */
    subscribers?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetActionSubscriber>[]>;
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
}
/**
 * The set of arguments for constructing a BudgetAction resource.
 */
export interface BudgetActionArgs {
    /**
     * The ID of the target account for budget. Will use current user's accountId by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The trigger threshold of the action. See Action Threshold.
     */
    actionThreshold: pulumi.Input<inputs.budgets.BudgetActionActionThreshold>;
    /**
     * The type of action. This defines the type of tasks that can be carried out by this action. This field also determines the format for definition. Valid values are `APPLY_IAM_POLICY`, `APPLY_SCP_POLICY`, and `RUN_SSM_DOCUMENTS`.
     */
    actionType: pulumi.Input<string>;
    /**
     * This specifies if the action needs manual or automatic approval. Valid values are `AUTOMATIC` and `MANUAL`.
     */
    approvalModel: pulumi.Input<string>;
    /**
     * The name of a budget.
     */
    budgetName: pulumi.Input<string>;
    /**
     * Specifies all of the type-specific parameters. See Definition.
     */
    definition: pulumi.Input<inputs.budgets.BudgetActionDefinition>;
    /**
     * The role passed for action execution and reversion. Roles and actions must be in the same account.
     */
    executionRoleArn: pulumi.Input<string>;
    /**
     * The type of a notification. Valid values are `ACTUAL` or `FORECASTED`.
     */
    notificationType: pulumi.Input<string>;
    /**
     * A list of subscribers. See Subscriber.
     */
    subscribers: pulumi.Input<pulumi.Input<inputs.budgets.BudgetActionSubscriber>[]>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
