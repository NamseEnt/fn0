import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Backup plan resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.Plan("example", {
 *     name: "my_example_backup_plan",
 *     rules: [{
 *         ruleName: "my_example_backup_rule",
 *         targetVaultName: test.name,
 *         schedule: "cron(0 12 * * ? *)",
 *         lifecycle: {
 *             deleteAfter: 14,
 *         },
 *     }],
 *     advancedBackupSettings: [{
 *         backupOptions: {
 *             WindowsVSS: "enabled",
 *         },
 *         resourceType: "EC2",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup Plan using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/plan:Plan test <id>
 * ```
 */
export declare class Plan extends pulumi.CustomResource {
    /**
     * Get an existing Plan resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PlanState, opts?: pulumi.CustomResourceOptions): Plan;
    /**
     * Returns true if the given object is an instance of Plan.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Plan;
    /**
     * An object that specifies backup options for each resource type.
     */
    readonly advancedBackupSettings: pulumi.Output<outputs.backup.PlanAdvancedBackupSetting[] | undefined>;
    /**
     * The ARN of the backup plan.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The display name of a backup plan.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A rule object that specifies a scheduled task that is used to back up a selection of resources.
     */
    readonly rules: pulumi.Output<outputs.backup.PlanRule[]>;
    /**
     * Metadata that you can assign to help organize the plans you create. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Unique, randomly generated, Unicode, UTF-8 encoded string that serves as the version ID of the backup plan.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a Plan resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PlanArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Plan resources.
 */
export interface PlanState {
    /**
     * An object that specifies backup options for each resource type.
     */
    advancedBackupSettings?: pulumi.Input<pulumi.Input<inputs.backup.PlanAdvancedBackupSetting>[]>;
    /**
     * The ARN of the backup plan.
     */
    arn?: pulumi.Input<string>;
    /**
     * The display name of a backup plan.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A rule object that specifies a scheduled task that is used to back up a selection of resources.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.backup.PlanRule>[]>;
    /**
     * Metadata that you can assign to help organize the plans you create. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Unique, randomly generated, Unicode, UTF-8 encoded string that serves as the version ID of the backup plan.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Plan resource.
 */
export interface PlanArgs {
    /**
     * An object that specifies backup options for each resource type.
     */
    advancedBackupSettings?: pulumi.Input<pulumi.Input<inputs.backup.PlanAdvancedBackupSetting>[]>;
    /**
     * The display name of a backup plan.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A rule object that specifies a scheduled task that is used to back up a selection of resources.
     */
    rules: pulumi.Input<pulumi.Input<inputs.backup.PlanRule>[]>;
    /**
     * Metadata that you can assign to help organize the plans you create. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
