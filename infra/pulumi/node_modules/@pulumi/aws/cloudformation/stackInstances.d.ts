import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudformation.StackInstances("example", {
 *     accounts: [
 *         "123456789012",
 *         "234567890123",
 *     ],
 *     regions: [
 *         "us-east-1",
 *         "us-west-2",
 *     ],
 *     stackSetName: exampleAwsCloudformationStackSet.name,
 * });
 * ```
 *
 * ### Example IAM Setup in Target Account
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const aWSCloudFormationStackSetExecutionRoleAssumeRolePolicy = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         effect: "Allow",
 *         principals: [{
 *             identifiers: [aWSCloudFormationStackSetAdministrationRole.arn],
 *             type: "AWS",
 *         }],
 *     }],
 * });
 * const aWSCloudFormationStackSetExecutionRole = new aws.iam.Role("AWSCloudFormationStackSetExecutionRole", {
 *     assumeRolePolicy: aWSCloudFormationStackSetExecutionRoleAssumeRolePolicy.then(aWSCloudFormationStackSetExecutionRoleAssumeRolePolicy => aWSCloudFormationStackSetExecutionRoleAssumeRolePolicy.json),
 *     name: "AWSCloudFormationStackSetExecutionRole",
 * });
 * // Documentation: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/stacksets-prereqs.html
 * // Additional IAM permissions necessary depend on the resources defined in the StackSet template
 * const aWSCloudFormationStackSetExecutionRoleMinimumExecutionPolicy = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: [
 *             "cloudformation:*",
 *             "s3:*",
 *             "sns:*",
 *         ],
 *         effect: "Allow",
 *         resources: ["*"],
 *     }],
 * });
 * const aWSCloudFormationStackSetExecutionRoleMinimumExecutionPolicyRolePolicy = new aws.iam.RolePolicy("AWSCloudFormationStackSetExecutionRole_MinimumExecutionPolicy", {
 *     name: "MinimumExecutionPolicy",
 *     policy: aWSCloudFormationStackSetExecutionRoleMinimumExecutionPolicy.then(aWSCloudFormationStackSetExecutionRoleMinimumExecutionPolicy => aWSCloudFormationStackSetExecutionRoleMinimumExecutionPolicy.json),
 *     role: aWSCloudFormationStackSetExecutionRole.name,
 * });
 * ```
 *
 * ### Example Deployment across Organizations account
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudformation.StackInstances("example", {
 *     deploymentTargets: {
 *         organizationalUnitIds: [exampleAwsOrganizationsOrganization.roots[0].id],
 *     },
 *     regions: [
 *         "us-west-2",
 *         "us-east-1",
 *     ],
 *     stackSetName: exampleAwsCloudformationStackSet.name,
 * });
 * ```
 *
 * ## Import
 *
 * Import CloudFormation stack instances that target OUs, using the stack set name, `call_as`, and "OU" separated by commas (`,`). For example:
 *
 * Using `pulumi import`, import CloudFormation stack instances using the stack set name and `call_as` separated by commas (`,`). If you are importing a stack instance targeting OUs, see the example below. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/stackInstances:StackInstances example example,SELF
 * ```
 * Using `pulumi import`, Import CloudFormation stack instances that target OUs, using the stack set name, `call_as`, and "OU" separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/stackInstances:StackInstances example example,SELF,OU
 * ```
 */
export declare class StackInstances extends pulumi.CustomResource {
    /**
     * Get an existing StackInstances resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StackInstancesState, opts?: pulumi.CustomResourceOptions): StackInstances;
    /**
     * Returns true if the given object is an instance of StackInstances.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StackInstances;
    /**
     * Accounts where you want to create stack instances in the specified `regions`. You can specify either `accounts` or `deploymentTargets`, but not both.
     */
    readonly accounts: pulumi.Output<string[]>;
    /**
     * Whether you are acting as an account administrator in the organization's management account or as a delegated administrator in a member account. Valid values: `SELF` (default), `DELEGATED_ADMIN`.
     */
    readonly callAs: pulumi.Output<string | undefined>;
    /**
     * AWS Organizations accounts for which to create stack instances in the `regions`. stack sets doesn't deploy stack instances to the organization management account, even if the organization management account is in your organization or in an OU in your organization. Drift detection is not possible for most of this argument. See deploymentTargets below.
     */
    readonly deploymentTargets: pulumi.Output<outputs.cloudformation.StackInstancesDeploymentTargets | undefined>;
    /**
     * Preferences for how AWS CloudFormation performs a stack set operation. See operationPreferences below.
     */
    readonly operationPreferences: pulumi.Output<outputs.cloudformation.StackInstancesOperationPreferences | undefined>;
    /**
     * Key-value map of input parameters to override from the stack set for these instances. This argument's drift detection is limited to the first account and region since each instance can have unique parameters.
     */
    readonly parameterOverrides: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Regions where you want to create stack instances in the specified `accounts`.
     */
    readonly regions: pulumi.Output<string[]>;
    /**
     * Whether to remove the stack instances from the stack set, but not delete the stacks. You can't reassociate a retained stack or add an existing, saved stack to a new stack set. To retain the stack, ensure `retainStacks = true` has been successfully applied _before_ an apply that would destroy the resource. Defaults to `false`.
     */
    readonly retainStacks: pulumi.Output<boolean | undefined>;
    /**
     * List of stack instances created from an organizational unit deployment target. This may not always be set depending on whether CloudFormation returns summaries for your configuration. See `stackInstanceSummaries`.
     */
    readonly stackInstanceSummaries: pulumi.Output<outputs.cloudformation.StackInstancesStackInstanceSummary[]>;
    /**
     * Name or unique ID of the stack set that the stack instance is associated with.
     */
    readonly stackSetId: pulumi.Output<string>;
    /**
     * Name of the stack set.
     *
     * The following arguments are optional:
     */
    readonly stackSetName: pulumi.Output<string>;
    /**
     * Create a StackInstances resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StackInstancesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StackInstances resources.
 */
export interface StackInstancesState {
    /**
     * Accounts where you want to create stack instances in the specified `regions`. You can specify either `accounts` or `deploymentTargets`, but not both.
     */
    accounts?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether you are acting as an account administrator in the organization's management account or as a delegated administrator in a member account. Valid values: `SELF` (default), `DELEGATED_ADMIN`.
     */
    callAs?: pulumi.Input<string>;
    /**
     * AWS Organizations accounts for which to create stack instances in the `regions`. stack sets doesn't deploy stack instances to the organization management account, even if the organization management account is in your organization or in an OU in your organization. Drift detection is not possible for most of this argument. See deploymentTargets below.
     */
    deploymentTargets?: pulumi.Input<inputs.cloudformation.StackInstancesDeploymentTargets>;
    /**
     * Preferences for how AWS CloudFormation performs a stack set operation. See operationPreferences below.
     */
    operationPreferences?: pulumi.Input<inputs.cloudformation.StackInstancesOperationPreferences>;
    /**
     * Key-value map of input parameters to override from the stack set for these instances. This argument's drift detection is limited to the first account and region since each instance can have unique parameters.
     */
    parameterOverrides?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Regions where you want to create stack instances in the specified `accounts`.
     */
    regions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to remove the stack instances from the stack set, but not delete the stacks. You can't reassociate a retained stack or add an existing, saved stack to a new stack set. To retain the stack, ensure `retainStacks = true` has been successfully applied _before_ an apply that would destroy the resource. Defaults to `false`.
     */
    retainStacks?: pulumi.Input<boolean>;
    /**
     * List of stack instances created from an organizational unit deployment target. This may not always be set depending on whether CloudFormation returns summaries for your configuration. See `stackInstanceSummaries`.
     */
    stackInstanceSummaries?: pulumi.Input<pulumi.Input<inputs.cloudformation.StackInstancesStackInstanceSummary>[]>;
    /**
     * Name or unique ID of the stack set that the stack instance is associated with.
     */
    stackSetId?: pulumi.Input<string>;
    /**
     * Name of the stack set.
     *
     * The following arguments are optional:
     */
    stackSetName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StackInstances resource.
 */
export interface StackInstancesArgs {
    /**
     * Accounts where you want to create stack instances in the specified `regions`. You can specify either `accounts` or `deploymentTargets`, but not both.
     */
    accounts?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether you are acting as an account administrator in the organization's management account or as a delegated administrator in a member account. Valid values: `SELF` (default), `DELEGATED_ADMIN`.
     */
    callAs?: pulumi.Input<string>;
    /**
     * AWS Organizations accounts for which to create stack instances in the `regions`. stack sets doesn't deploy stack instances to the organization management account, even if the organization management account is in your organization or in an OU in your organization. Drift detection is not possible for most of this argument. See deploymentTargets below.
     */
    deploymentTargets?: pulumi.Input<inputs.cloudformation.StackInstancesDeploymentTargets>;
    /**
     * Preferences for how AWS CloudFormation performs a stack set operation. See operationPreferences below.
     */
    operationPreferences?: pulumi.Input<inputs.cloudformation.StackInstancesOperationPreferences>;
    /**
     * Key-value map of input parameters to override from the stack set for these instances. This argument's drift detection is limited to the first account and region since each instance can have unique parameters.
     */
    parameterOverrides?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Regions where you want to create stack instances in the specified `accounts`.
     */
    regions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to remove the stack instances from the stack set, but not delete the stacks. You can't reassociate a retained stack or add an existing, saved stack to a new stack set. To retain the stack, ensure `retainStacks = true` has been successfully applied _before_ an apply that would destroy the resource. Defaults to `false`.
     */
    retainStacks?: pulumi.Input<boolean>;
    /**
     * Name of the stack set.
     *
     * The following arguments are optional:
     */
    stackSetName: pulumi.Input<string>;
}
