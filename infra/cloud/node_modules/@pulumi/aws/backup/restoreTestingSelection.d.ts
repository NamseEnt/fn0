import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Backup Restore Testing Selection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.RestoreTestingSelection("example", {
 *     name: "ec2_selection",
 *     restoreTestingPlanName: exampleAwsBackupRestoreTestingPlan.name,
 *     protectedResourceType: "EC2",
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     protectedResourceArns: ["*"],
 * });
 * ```
 *
 * ### Advanced Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.RestoreTestingSelection("example", {
 *     name: "ec2_selection",
 *     restoreTestingPlanName: exampleAwsBackupRestoreTestingPlan.name,
 *     protectedResourceType: "EC2",
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     protectedResourceConditions: {
 *         stringEquals: [{
 *             key: "aws:ResourceTag/backup",
 *             value: "true",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup Restore Testing Selection using `name:restore_testing_plan_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/restoreTestingSelection:RestoreTestingSelection example restore_testing_selection_12345678:restore_testing_plan_12345678
 * ```
 */
export declare class RestoreTestingSelection extends pulumi.CustomResource {
    /**
     * Get an existing RestoreTestingSelection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RestoreTestingSelectionState, opts?: pulumi.CustomResourceOptions): RestoreTestingSelection;
    /**
     * Returns true if the given object is an instance of RestoreTestingSelection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RestoreTestingSelection;
    /**
     * The ARN of the IAM role.
     */
    readonly iamRoleArn: pulumi.Output<string>;
    /**
     * The name of the backup restore testing selection.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ARNs for the protected resources.
     */
    readonly protectedResourceArns: pulumi.Output<string[]>;
    /**
     * The conditions for the protected resource.
     */
    readonly protectedResourceConditions: pulumi.Output<outputs.backup.RestoreTestingSelectionProtectedResourceConditions | undefined>;
    /**
     * The type of the protected resource.
     */
    readonly protectedResourceType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Override certain restore metadata keys. See the complete list of [restore testing inferred metadata](https://docs.aws.amazon.com/aws-backup/latest/devguide/restore-testing-inferred-metadata.html) .
     */
    readonly restoreMetadataOverrides: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The name of the restore testing plan.
     */
    readonly restoreTestingPlanName: pulumi.Output<string>;
    /**
     * The amount of hours available to run a validation script on the data. Valid range is `1` to `168`.
     */
    readonly validationWindowHours: pulumi.Output<number>;
    /**
     * Create a RestoreTestingSelection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RestoreTestingSelectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RestoreTestingSelection resources.
 */
export interface RestoreTestingSelectionState {
    /**
     * The ARN of the IAM role.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * The name of the backup restore testing selection.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARNs for the protected resources.
     */
    protectedResourceArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The conditions for the protected resource.
     */
    protectedResourceConditions?: pulumi.Input<inputs.backup.RestoreTestingSelectionProtectedResourceConditions>;
    /**
     * The type of the protected resource.
     */
    protectedResourceType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Override certain restore metadata keys. See the complete list of [restore testing inferred metadata](https://docs.aws.amazon.com/aws-backup/latest/devguide/restore-testing-inferred-metadata.html) .
     */
    restoreMetadataOverrides?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the restore testing plan.
     */
    restoreTestingPlanName?: pulumi.Input<string>;
    /**
     * The amount of hours available to run a validation script on the data. Valid range is `1` to `168`.
     */
    validationWindowHours?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a RestoreTestingSelection resource.
 */
export interface RestoreTestingSelectionArgs {
    /**
     * The ARN of the IAM role.
     */
    iamRoleArn: pulumi.Input<string>;
    /**
     * The name of the backup restore testing selection.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARNs for the protected resources.
     */
    protectedResourceArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The conditions for the protected resource.
     */
    protectedResourceConditions?: pulumi.Input<inputs.backup.RestoreTestingSelectionProtectedResourceConditions>;
    /**
     * The type of the protected resource.
     */
    protectedResourceType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Override certain restore metadata keys. See the complete list of [restore testing inferred metadata](https://docs.aws.amazon.com/aws-backup/latest/devguide/restore-testing-inferred-metadata.html) .
     */
    restoreMetadataOverrides?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the restore testing plan.
     */
    restoreTestingPlanName: pulumi.Input<string>;
    /**
     * The amount of hours available to run a validation script on the data. Valid range is `1` to `168`.
     */
    validationWindowHours?: pulumi.Input<number>;
}
