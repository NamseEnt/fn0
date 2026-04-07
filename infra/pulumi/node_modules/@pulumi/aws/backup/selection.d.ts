import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages selection conditions for AWS Backup plan resources.
 *
 * ## Example Usage
 *
 * ### IAM Role
 *
 * > For more information about creating and managing IAM Roles for backups and restores, see the [AWS Backup Developer Guide](https://docs.aws.amazon.com/aws-backup/latest/devguide/iam-service-roles.html).
 *
 * The below example creates an IAM role with the default managed IAM Policy for allowing AWS Backup to create backups.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["backup.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const example = new aws.iam.Role("example", {
 *     name: "example",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("example", {
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AWSBackupServiceRolePolicyForBackup",
 *     role: example.name,
 * });
 * const exampleSelection = new aws.backup.Selection("example", {iamRoleArn: example.arn});
 * ```
 *
 * ### Selecting Backups By Tag
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.Selection("example", {
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     name: "my_example_backup_selection",
 *     planId: exampleAwsBackupPlan.id,
 *     selectionTags: [{
 *         type: "STRINGEQUALS",
 *         key: "foo",
 *         value: "bar",
 *     }],
 * });
 * ```
 *
 * ### Selecting Backups By Conditions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.Selection("example", {
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     name: "my_example_backup_selection",
 *     planId: exampleAwsBackupPlan.id,
 *     resources: ["*"],
 *     conditions: [{
 *         stringEquals: [{
 *             key: "aws:ResourceTag/Component",
 *             value: "rds",
 *         }],
 *         stringLikes: [{
 *             key: "aws:ResourceTag/Application",
 *             value: "app*",
 *         }],
 *         stringNotEquals: [{
 *             key: "aws:ResourceTag/Backup",
 *             value: "false",
 *         }],
 *         stringNotLikes: [{
 *             key: "aws:ResourceTag/Environment",
 *             value: "test*",
 *         }],
 *     }],
 * });
 * ```
 *
 * ### Selecting Backups By Resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.Selection("example", {
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     name: "my_example_backup_selection",
 *     planId: exampleAwsBackupPlan.id,
 *     resources: [
 *         exampleAwsDbInstance.arn,
 *         exampleAwsEbsVolume.arn,
 *         exampleAwsEfsFileSystem.arn,
 *     ],
 * });
 * ```
 *
 * ### Selecting Backups By Not Resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.Selection("example", {
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     name: "my_example_backup_selection",
 *     planId: exampleAwsBackupPlan.id,
 *     notResources: [
 *         exampleAwsDbInstance.arn,
 *         exampleAwsEbsVolume.arn,
 *         exampleAwsEfsFileSystem.arn,
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup selection using the role plan_id and id separated by `|`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/selection:Selection example plan-id|selection-id
 * ```
 */
export declare class Selection extends pulumi.CustomResource {
    /**
     * Get an existing Selection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SelectionState, opts?: pulumi.CustomResourceOptions): Selection;
    /**
     * Returns true if the given object is an instance of Selection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Selection;
    /**
     * Condition-based filters used to specify sets of resources for a backup plan. See below for details.
     */
    readonly conditions: pulumi.Output<outputs.backup.SelectionCondition[]>;
    /**
     * The ARN of the IAM role that AWS Backup uses to authenticate when restoring and backing up the target resource. See the [AWS Backup Developer Guide](https://docs.aws.amazon.com/aws-backup/latest/devguide/access-control.html#managed-policies) for additional information about using AWS managed policies or creating custom policies attached to the IAM role.
     */
    readonly iamRoleArn: pulumi.Output<string>;
    /**
     * The display name of a resource selection document.
     */
    readonly name: pulumi.Output<string>;
    /**
     * An array of strings that either contain Amazon Resource Names (ARNs) or match patterns of resources to exclude from a backup plan.
     */
    readonly notResources: pulumi.Output<string[]>;
    /**
     * The backup plan ID to be associated with the selection of resources.
     */
    readonly planId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An array of strings that either contain Amazon Resource Names (ARNs) or match patterns of resources to assign to a backup plan.
     */
    readonly resources: pulumi.Output<string[] | undefined>;
    /**
     * Tag-based conditions used to specify a set of resources to assign to a backup plan. See below for details.
     */
    readonly selectionTags: pulumi.Output<outputs.backup.SelectionSelectionTag[] | undefined>;
    /**
     * Create a Selection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SelectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Selection resources.
 */
export interface SelectionState {
    /**
     * Condition-based filters used to specify sets of resources for a backup plan. See below for details.
     */
    conditions?: pulumi.Input<pulumi.Input<inputs.backup.SelectionCondition>[]>;
    /**
     * The ARN of the IAM role that AWS Backup uses to authenticate when restoring and backing up the target resource. See the [AWS Backup Developer Guide](https://docs.aws.amazon.com/aws-backup/latest/devguide/access-control.html#managed-policies) for additional information about using AWS managed policies or creating custom policies attached to the IAM role.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * The display name of a resource selection document.
     */
    name?: pulumi.Input<string>;
    /**
     * An array of strings that either contain Amazon Resource Names (ARNs) or match patterns of resources to exclude from a backup plan.
     */
    notResources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The backup plan ID to be associated with the selection of resources.
     */
    planId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An array of strings that either contain Amazon Resource Names (ARNs) or match patterns of resources to assign to a backup plan.
     */
    resources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Tag-based conditions used to specify a set of resources to assign to a backup plan. See below for details.
     */
    selectionTags?: pulumi.Input<pulumi.Input<inputs.backup.SelectionSelectionTag>[]>;
}
/**
 * The set of arguments for constructing a Selection resource.
 */
export interface SelectionArgs {
    /**
     * Condition-based filters used to specify sets of resources for a backup plan. See below for details.
     */
    conditions?: pulumi.Input<pulumi.Input<inputs.backup.SelectionCondition>[]>;
    /**
     * The ARN of the IAM role that AWS Backup uses to authenticate when restoring and backing up the target resource. See the [AWS Backup Developer Guide](https://docs.aws.amazon.com/aws-backup/latest/devguide/access-control.html#managed-policies) for additional information about using AWS managed policies or creating custom policies attached to the IAM role.
     */
    iamRoleArn: pulumi.Input<string>;
    /**
     * The display name of a resource selection document.
     */
    name?: pulumi.Input<string>;
    /**
     * An array of strings that either contain Amazon Resource Names (ARNs) or match patterns of resources to exclude from a backup plan.
     */
    notResources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The backup plan ID to be associated with the selection of resources.
     */
    planId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An array of strings that either contain Amazon Resource Names (ARNs) or match patterns of resources to assign to a backup plan.
     */
    resources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Tag-based conditions used to specify a set of resources to assign to a backup plan. See below for details.
     */
    selectionTags?: pulumi.Input<pulumi.Input<inputs.backup.SelectionSelectionTag>[]>;
}
