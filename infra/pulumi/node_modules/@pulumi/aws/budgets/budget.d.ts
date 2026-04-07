import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a budgets budget resource. Budgets use the cost visualization provided by Cost Explorer to show you the status of your budgets, to provide forecasts of your estimated costs, and to track your AWS usage, including your free tier usage.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ec2 = new aws.budgets.Budget("ec2", {
 *     name: "budget-ec2-monthly",
 *     budgetType: "COST",
 *     limitAmount: "1200",
 *     limitUnit: "USD",
 *     timePeriodEnd: "2087-06-15_00:00",
 *     timePeriodStart: "2017-07-01_00:00",
 *     timeUnit: "MONTHLY",
 *     costFilters: [{
 *         name: "Service",
 *         values: ["Amazon Elastic Compute Cloud - Compute"],
 *     }],
 *     notifications: [{
 *         comparisonOperator: "GREATER_THAN",
 *         threshold: 100,
 *         thresholdType: "PERCENTAGE",
 *         notificationType: "FORECASTED",
 *         subscriberEmailAddresses: ["test@example.com"],
 *     }],
 *     tags: {
 *         Tag1: "Value1",
 *         Tag2: "Value2",
 *     },
 * });
 * ```
 *
 * Create a budget for *$100*.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cost = new aws.budgets.Budget("cost", {
 *     budgetType: "COST",
 *     limitAmount: "100",
 *     limitUnit: "USD",
 * });
 * ```
 *
 * Create a budget with planned budget limits.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cost = new aws.budgets.Budget("cost", {plannedLimits: [
 *     {
 *         startTime: "2017-07-01_00:00",
 *         amount: "100",
 *         unit: "USD",
 *     },
 *     {
 *         startTime: "2017-08-01_00:00",
 *         amount: "200",
 *         unit: "USD",
 *     },
 * ]});
 * ```
 *
 * Create a budget for s3 with a limit of *3 GB* of storage.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const s3 = new aws.budgets.Budget("s3", {
 *     budgetType: "USAGE",
 *     limitAmount: "3",
 *     limitUnit: "GB",
 * });
 * ```
 *
 * Create a Savings Plan Utilization Budget
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const savingsPlanUtilization = new aws.budgets.Budget("savings_plan_utilization", {
 *     budgetType: "SAVINGS_PLANS_UTILIZATION",
 *     limitAmount: "100.0",
 *     limitUnit: "PERCENTAGE",
 *     costTypes: {
 *         includeCredit: false,
 *         includeDiscount: false,
 *         includeOtherSubscription: false,
 *         includeRecurring: false,
 *         includeRefund: false,
 *         includeSubscription: true,
 *         includeSupport: false,
 *         includeTax: false,
 *         includeUpfront: false,
 *         useBlended: false,
 *     },
 * });
 * ```
 *
 * Create a RI Utilization Budget
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const riUtilization = new aws.budgets.Budget("ri_utilization", {
 *     budgetType: "RI_UTILIZATION",
 *     limitAmount: "100.0",
 *     limitUnit: "PERCENTAGE",
 *     costTypes: {
 *         includeCredit: false,
 *         includeDiscount: false,
 *         includeOtherSubscription: false,
 *         includeRecurring: false,
 *         includeRefund: false,
 *         includeSubscription: true,
 *         includeSupport: false,
 *         includeTax: false,
 *         includeUpfront: false,
 *         useBlended: false,
 *     },
 *     costFilters: [{
 *         name: "Service",
 *         values: ["Amazon Relational Database Service"],
 *     }],
 * });
 * ```
 *
 * Create a cost filter using resource tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cost = new aws.budgets.Budget("cost", {costFilters: [{
 *     name: "TagKeyValue",
 *     values: [
 *         "aws:createdBy$Pulumi",
 *         "user:business-unit$human_resources",
 *     ],
 * }]});
 * ```
 *
 * Create a cost filter using resource tags, obtaining the tag value from a variable
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cost = new aws.budgets.Budget("cost", {costFilters: [{
 *     name: "TagKeyValue",
 *     values: [`TagKey${"$"}${tagValue}`],
 * }]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import budgets using `AccountID:BudgetName`. For example:
 *
 * ```sh
 * $ pulumi import aws:budgets/budget:Budget myBudget 123456789012:myBudget
 * ```
 */
export declare class Budget extends pulumi.CustomResource {
    /**
     * Get an existing Budget resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BudgetState, opts?: pulumi.CustomResourceOptions): Budget;
    /**
     * Returns true if the given object is an instance of Budget.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Budget;
    /**
     * The ID of the target account for budget. Will use current user's accountId by default if omitted.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The ARN of the budget.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Object containing AutoAdjustData which determines the budget amount for an auto-adjusting budget.
     */
    readonly autoAdjustData: pulumi.Output<outputs.budgets.BudgetAutoAdjustData | undefined>;
    /**
     * ARN of the billing view.
     */
    readonly billingViewArn: pulumi.Output<string | undefined>;
    /**
     * Whether this budget tracks monetary cost or usage.
     */
    readonly budgetType: pulumi.Output<string>;
    /**
     * A list of CostFilter name/values pair to apply to budget.
     */
    readonly costFilters: pulumi.Output<outputs.budgets.BudgetCostFilter[]>;
    /**
     * Object containing CostTypes The types of cost included in a budget, such as tax and subscriptions.
     */
    readonly costTypes: pulumi.Output<outputs.budgets.BudgetCostTypes>;
    /**
     * The amount of cost or usage being measured for a budget.
     */
    readonly limitAmount: pulumi.Output<string>;
    /**
     * The unit of measurement used for the budget forecast, actual spend, or budget threshold, such as dollars or GB. See [Spend](http://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/data-type-spend.html) documentation.
     */
    readonly limitUnit: pulumi.Output<string>;
    /**
     * The name of a budget. Unique within accounts.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The prefix of the name of a budget. Unique within accounts.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Object containing Budget Notifications. Can be used multiple times to define more than one budget notification.
     */
    readonly notifications: pulumi.Output<outputs.budgets.BudgetNotification[] | undefined>;
    /**
     * Object containing Planned Budget Limits. Can be used multiple times to plan more than one budget limit. See [PlannedBudgetLimits](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-PlannedBudgetLimits) documentation.
     */
    readonly plannedLimits: pulumi.Output<outputs.budgets.BudgetPlannedLimit[] | undefined>;
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
     * The end of the time period covered by the budget. There are no restrictions on the end date. Format: `2017-01-01_12:00`.
     */
    readonly timePeriodEnd: pulumi.Output<string | undefined>;
    /**
     * The start of the time period covered by the budget. If you don't specify a start date, AWS defaults to the start of your chosen time period. The start date must come before the end date. Format: `2017-01-01_12:00`.
     *
     * For more detailed documentation about each argument, refer to the [AWS official
     * documentation](http://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/data-type-budget.html).
     */
    readonly timePeriodStart: pulumi.Output<string>;
    /**
     * The length of time until a budget resets the actual and forecasted spend. Valid values: `MONTHLY`, `QUARTERLY`, `ANNUALLY`, and `DAILY`.
     *
     * The following arguments are optional:
     */
    readonly timeUnit: pulumi.Output<string>;
    /**
     * Create a Budget resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BudgetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Budget resources.
 */
export interface BudgetState {
    /**
     * The ID of the target account for budget. Will use current user's accountId by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The ARN of the budget.
     */
    arn?: pulumi.Input<string>;
    /**
     * Object containing AutoAdjustData which determines the budget amount for an auto-adjusting budget.
     */
    autoAdjustData?: pulumi.Input<inputs.budgets.BudgetAutoAdjustData>;
    /**
     * ARN of the billing view.
     */
    billingViewArn?: pulumi.Input<string>;
    /**
     * Whether this budget tracks monetary cost or usage.
     */
    budgetType?: pulumi.Input<string>;
    /**
     * A list of CostFilter name/values pair to apply to budget.
     */
    costFilters?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetCostFilter>[]>;
    /**
     * Object containing CostTypes The types of cost included in a budget, such as tax and subscriptions.
     */
    costTypes?: pulumi.Input<inputs.budgets.BudgetCostTypes>;
    /**
     * The amount of cost or usage being measured for a budget.
     */
    limitAmount?: pulumi.Input<string>;
    /**
     * The unit of measurement used for the budget forecast, actual spend, or budget threshold, such as dollars or GB. See [Spend](http://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/data-type-spend.html) documentation.
     */
    limitUnit?: pulumi.Input<string>;
    /**
     * The name of a budget. Unique within accounts.
     */
    name?: pulumi.Input<string>;
    /**
     * The prefix of the name of a budget. Unique within accounts.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Object containing Budget Notifications. Can be used multiple times to define more than one budget notification.
     */
    notifications?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetNotification>[]>;
    /**
     * Object containing Planned Budget Limits. Can be used multiple times to plan more than one budget limit. See [PlannedBudgetLimits](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-PlannedBudgetLimits) documentation.
     */
    plannedLimits?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetPlannedLimit>[]>;
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
     * The end of the time period covered by the budget. There are no restrictions on the end date. Format: `2017-01-01_12:00`.
     */
    timePeriodEnd?: pulumi.Input<string>;
    /**
     * The start of the time period covered by the budget. If you don't specify a start date, AWS defaults to the start of your chosen time period. The start date must come before the end date. Format: `2017-01-01_12:00`.
     *
     * For more detailed documentation about each argument, refer to the [AWS official
     * documentation](http://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/data-type-budget.html).
     */
    timePeriodStart?: pulumi.Input<string>;
    /**
     * The length of time until a budget resets the actual and forecasted spend. Valid values: `MONTHLY`, `QUARTERLY`, `ANNUALLY`, and `DAILY`.
     *
     * The following arguments are optional:
     */
    timeUnit?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Budget resource.
 */
export interface BudgetArgs {
    /**
     * The ID of the target account for budget. Will use current user's accountId by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Object containing AutoAdjustData which determines the budget amount for an auto-adjusting budget.
     */
    autoAdjustData?: pulumi.Input<inputs.budgets.BudgetAutoAdjustData>;
    /**
     * ARN of the billing view.
     */
    billingViewArn?: pulumi.Input<string>;
    /**
     * Whether this budget tracks monetary cost or usage.
     */
    budgetType: pulumi.Input<string>;
    /**
     * A list of CostFilter name/values pair to apply to budget.
     */
    costFilters?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetCostFilter>[]>;
    /**
     * Object containing CostTypes The types of cost included in a budget, such as tax and subscriptions.
     */
    costTypes?: pulumi.Input<inputs.budgets.BudgetCostTypes>;
    /**
     * The amount of cost or usage being measured for a budget.
     */
    limitAmount?: pulumi.Input<string>;
    /**
     * The unit of measurement used for the budget forecast, actual spend, or budget threshold, such as dollars or GB. See [Spend](http://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/data-type-spend.html) documentation.
     */
    limitUnit?: pulumi.Input<string>;
    /**
     * The name of a budget. Unique within accounts.
     */
    name?: pulumi.Input<string>;
    /**
     * The prefix of the name of a budget. Unique within accounts.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Object containing Budget Notifications. Can be used multiple times to define more than one budget notification.
     */
    notifications?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetNotification>[]>;
    /**
     * Object containing Planned Budget Limits. Can be used multiple times to plan more than one budget limit. See [PlannedBudgetLimits](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-PlannedBudgetLimits) documentation.
     */
    plannedLimits?: pulumi.Input<pulumi.Input<inputs.budgets.BudgetPlannedLimit>[]>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The end of the time period covered by the budget. There are no restrictions on the end date. Format: `2017-01-01_12:00`.
     */
    timePeriodEnd?: pulumi.Input<string>;
    /**
     * The start of the time period covered by the budget. If you don't specify a start date, AWS defaults to the start of your chosen time period. The start date must come before the end date. Format: `2017-01-01_12:00`.
     *
     * For more detailed documentation about each argument, refer to the [AWS official
     * documentation](http://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/data-type-budget.html).
     */
    timePeriodStart?: pulumi.Input<string>;
    /**
     * The length of time until a budget resets the actual and forecasted spend. Valid values: `MONTHLY`, `QUARTERLY`, `ANNUALLY`, and `DAILY`.
     *
     * The following arguments are optional:
     */
    timeUnit: pulumi.Input<string>;
}
