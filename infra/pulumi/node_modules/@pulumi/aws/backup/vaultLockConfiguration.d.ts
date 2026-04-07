import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Backup vault lock configuration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.backup.VaultLockConfiguration("test", {
 *     backupVaultName: "example_backup_vault",
 *     changeableForDays: 3,
 *     maxRetentionDays: 1200,
 *     minRetentionDays: 7,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup vault lock configuration using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/vaultLockConfiguration:VaultLockConfiguration test TestVault
 * ```
 */
export declare class VaultLockConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing VaultLockConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VaultLockConfigurationState, opts?: pulumi.CustomResourceOptions): VaultLockConfiguration;
    /**
     * Returns true if the given object is an instance of VaultLockConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VaultLockConfiguration;
    /**
     * The ARN of the vault.
     */
    readonly backupVaultArn: pulumi.Output<string>;
    /**
     * Name of the backup vault to add a lock configuration for.
     */
    readonly backupVaultName: pulumi.Output<string>;
    /**
     * The number of days before the lock date. If omitted creates a vault lock in `governance` mode, otherwise it will create a vault lock in `compliance` mode.
     */
    readonly changeableForDays: pulumi.Output<number | undefined>;
    /**
     * The maximum retention period that the vault retains its recovery points.
     */
    readonly maxRetentionDays: pulumi.Output<number | undefined>;
    /**
     * The minimum retention period that the vault retains its recovery points.
     */
    readonly minRetentionDays: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a VaultLockConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VaultLockConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VaultLockConfiguration resources.
 */
export interface VaultLockConfigurationState {
    /**
     * The ARN of the vault.
     */
    backupVaultArn?: pulumi.Input<string>;
    /**
     * Name of the backup vault to add a lock configuration for.
     */
    backupVaultName?: pulumi.Input<string>;
    /**
     * The number of days before the lock date. If omitted creates a vault lock in `governance` mode, otherwise it will create a vault lock in `compliance` mode.
     */
    changeableForDays?: pulumi.Input<number>;
    /**
     * The maximum retention period that the vault retains its recovery points.
     */
    maxRetentionDays?: pulumi.Input<number>;
    /**
     * The minimum retention period that the vault retains its recovery points.
     */
    minRetentionDays?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VaultLockConfiguration resource.
 */
export interface VaultLockConfigurationArgs {
    /**
     * Name of the backup vault to add a lock configuration for.
     */
    backupVaultName: pulumi.Input<string>;
    /**
     * The number of days before the lock date. If omitted creates a vault lock in `governance` mode, otherwise it will create a vault lock in `compliance` mode.
     */
    changeableForDays?: pulumi.Input<number>;
    /**
     * The maximum retention period that the vault retains its recovery points.
     */
    maxRetentionDays?: pulumi.Input<number>;
    /**
     * The minimum retention period that the vault retains its recovery points.
     */
    minRetentionDays?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
