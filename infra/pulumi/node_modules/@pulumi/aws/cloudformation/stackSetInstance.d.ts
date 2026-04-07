import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a CloudFormation StackSet Instance. Instances are managed in the account and region of the StackSet after the target account permissions have been configured. Additional information about StackSets can be found in the [AWS CloudFormation User Guide](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/what-is-cfnstacksets.html).
 *
 * > **NOTE:** All target accounts must have an IAM Role created that matches the name of the execution role configured in the StackSet (the `executionRoleName` argument in the `aws.cloudformation.StackSet` resource) in a trust relationship with the administrative account or administration IAM Role. The execution role must have appropriate permissions to manage resources defined in the template along with those required for StackSets to operate. See the [AWS CloudFormation User Guide](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/stacksets-prereqs.html) for more details.
 *
 * > **NOTE:** To retain the Stack during resource destroy, ensure `retainStack` has been set to `true` in the state first. This must be completed _before_ a deployment that would destroy the resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudformation.StackSetInstance("example", {
 *     accountId: "123456789012",
 *     stackSetInstanceRegion: "us-east-1",
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
 * const example = new aws.cloudformation.StackSetInstance("example", {
 *     deploymentTargets: {
 *         organizationalUnitIds: [exampleAwsOrganizationsOrganization.roots[0].id],
 *     },
 *     stackSetInstanceRegion: "us-east-1",
 *     stackSetName: exampleAwsCloudformationStackSet.name,
 * });
 * ```
 *
 * ## Import
 *
 * Import CloudFormation StackSet Instances that target AWS Organizational Units using the StackSet name, a slash (`/`) separated list of organizational unit IDs, and target AWS Region separated by commas (`,`). For example:
 *
 * Import CloudFormation StackSet Instances when acting a delegated administrator in a member account using the StackSet name, target AWS account ID or slash (`/`) separated list of organizational unit IDs, target AWS Region and `call_as` value separated by commas (`,`). For example:
 *
 * Using `pulumi import`, import CloudFormation StackSet Instances that target an AWS Account ID using the StackSet name, target AWS account ID, and target AWS Region separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/stackSetInstance:StackSetInstance example example,123456789012,us-east-1
 * ```
 * Using `pulumi import`, import CloudFormation StackSet Instances that target AWS Organizational Units using the StackSet name, a slash (`/`) separated list of organizational unit IDs, and target AWS Region separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/stackSetInstance:StackSetInstance example example,ou-sdas-123123123/ou-sdas-789789789,us-east-1
 * ```
 * Using `pulumi import`, import CloudFormation StackSet Instances when acting a delegated administrator in a member account using the StackSet name, target AWS account ID or slash (`/`) separated list of organizational unit IDs, target AWS Region and `call_as` value separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/stackSetInstance:StackSetInstance example example,ou-sdas-123123123/ou-sdas-789789789,us-east-1,DELEGATED_ADMIN
 * ```
 */
export declare class StackSetInstance extends pulumi.CustomResource {
    /**
     * Get an existing StackSetInstance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StackSetInstanceState, opts?: pulumi.CustomResourceOptions): StackSetInstance;
    /**
     * Returns true if the given object is an instance of StackSetInstance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StackSetInstance;
    /**
     * Target AWS Account ID to create a Stack based on the StackSet. Defaults to current account.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Specifies whether you are acting as an account administrator in the organization's management account or as a delegated administrator in a member account. Valid values: `SELF` (default), `DELEGATED_ADMIN`.
     */
    readonly callAs: pulumi.Output<string | undefined>;
    /**
     * AWS Organizations accounts to which StackSets deploys. StackSets doesn't deploy stack instances to the organization management account, even if the organization management account is in your organization or in an OU in your organization. Drift detection is not possible for this argument. See deploymentTargets below.
     */
    readonly deploymentTargets: pulumi.Output<outputs.cloudformation.StackSetInstanceDeploymentTargets | undefined>;
    /**
     * Preferences for how AWS CloudFormation performs a stack set operation.
     */
    readonly operationPreferences: pulumi.Output<outputs.cloudformation.StackSetInstanceOperationPreferences | undefined>;
    /**
     * Organizational unit ID in which the stack is deployed.
     */
    readonly organizationalUnitId: pulumi.Output<string>;
    /**
     * Key-value map of input parameters to override from the StackSet for this Instance.
     */
    readonly parameterOverrides: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Target AWS Region to create a Stack based on the StackSet. Defaults to current region. Use `stackSetInstanceRegion` instead.
     *
     * @deprecated region is deprecated. Use stackSetInstanceRegion instead.
     */
    readonly region: pulumi.Output<string>;
    /**
     * During resource destroy, remove Instance from StackSet while keeping the Stack and its associated resources. Must be enabled in the state _before_ destroy operation to take effect. You cannot reassociate a retained Stack or add an existing, saved Stack to a new StackSet. Defaults to `false`.
     */
    readonly retainStack: pulumi.Output<boolean | undefined>;
    /**
     * Stack identifier.
     */
    readonly stackId: pulumi.Output<string>;
    /**
     * List of stack instances created from an organizational unit deployment target. This will only be populated when `deploymentTargets` is set. See `stackInstanceSummaries`.
     */
    readonly stackInstanceSummaries: pulumi.Output<outputs.cloudformation.StackSetInstanceStackInstanceSummary[]>;
    /**
     * Target AWS Region to create a Stack based on the StackSet. Defaults to current region.
     */
    readonly stackSetInstanceRegion: pulumi.Output<string>;
    /**
     * Name of the StackSet.
     */
    readonly stackSetName: pulumi.Output<string>;
    /**
     * Create a StackSetInstance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StackSetInstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StackSetInstance resources.
 */
export interface StackSetInstanceState {
    /**
     * Target AWS Account ID to create a Stack based on the StackSet. Defaults to current account.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Specifies whether you are acting as an account administrator in the organization's management account or as a delegated administrator in a member account. Valid values: `SELF` (default), `DELEGATED_ADMIN`.
     */
    callAs?: pulumi.Input<string>;
    /**
     * AWS Organizations accounts to which StackSets deploys. StackSets doesn't deploy stack instances to the organization management account, even if the organization management account is in your organization or in an OU in your organization. Drift detection is not possible for this argument. See deploymentTargets below.
     */
    deploymentTargets?: pulumi.Input<inputs.cloudformation.StackSetInstanceDeploymentTargets>;
    /**
     * Preferences for how AWS CloudFormation performs a stack set operation.
     */
    operationPreferences?: pulumi.Input<inputs.cloudformation.StackSetInstanceOperationPreferences>;
    /**
     * Organizational unit ID in which the stack is deployed.
     */
    organizationalUnitId?: pulumi.Input<string>;
    /**
     * Key-value map of input parameters to override from the StackSet for this Instance.
     */
    parameterOverrides?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Target AWS Region to create a Stack based on the StackSet. Defaults to current region. Use `stackSetInstanceRegion` instead.
     *
     * @deprecated region is deprecated. Use stackSetInstanceRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * During resource destroy, remove Instance from StackSet while keeping the Stack and its associated resources. Must be enabled in the state _before_ destroy operation to take effect. You cannot reassociate a retained Stack or add an existing, saved Stack to a new StackSet. Defaults to `false`.
     */
    retainStack?: pulumi.Input<boolean>;
    /**
     * Stack identifier.
     */
    stackId?: pulumi.Input<string>;
    /**
     * List of stack instances created from an organizational unit deployment target. This will only be populated when `deploymentTargets` is set. See `stackInstanceSummaries`.
     */
    stackInstanceSummaries?: pulumi.Input<pulumi.Input<inputs.cloudformation.StackSetInstanceStackInstanceSummary>[]>;
    /**
     * Target AWS Region to create a Stack based on the StackSet. Defaults to current region.
     */
    stackSetInstanceRegion?: pulumi.Input<string>;
    /**
     * Name of the StackSet.
     */
    stackSetName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StackSetInstance resource.
 */
export interface StackSetInstanceArgs {
    /**
     * Target AWS Account ID to create a Stack based on the StackSet. Defaults to current account.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Specifies whether you are acting as an account administrator in the organization's management account or as a delegated administrator in a member account. Valid values: `SELF` (default), `DELEGATED_ADMIN`.
     */
    callAs?: pulumi.Input<string>;
    /**
     * AWS Organizations accounts to which StackSets deploys. StackSets doesn't deploy stack instances to the organization management account, even if the organization management account is in your organization or in an OU in your organization. Drift detection is not possible for this argument. See deploymentTargets below.
     */
    deploymentTargets?: pulumi.Input<inputs.cloudformation.StackSetInstanceDeploymentTargets>;
    /**
     * Preferences for how AWS CloudFormation performs a stack set operation.
     */
    operationPreferences?: pulumi.Input<inputs.cloudformation.StackSetInstanceOperationPreferences>;
    /**
     * Key-value map of input parameters to override from the StackSet for this Instance.
     */
    parameterOverrides?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Target AWS Region to create a Stack based on the StackSet. Defaults to current region. Use `stackSetInstanceRegion` instead.
     *
     * @deprecated region is deprecated. Use stackSetInstanceRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * During resource destroy, remove Instance from StackSet while keeping the Stack and its associated resources. Must be enabled in the state _before_ destroy operation to take effect. You cannot reassociate a retained Stack or add an existing, saved Stack to a new StackSet. Defaults to `false`.
     */
    retainStack?: pulumi.Input<boolean>;
    /**
     * Target AWS Region to create a Stack based on the StackSet. Defaults to current region.
     */
    stackSetInstanceRegion?: pulumi.Input<string>;
    /**
     * Name of the StackSet.
     */
    stackSetName: pulumi.Input<string>;
}
