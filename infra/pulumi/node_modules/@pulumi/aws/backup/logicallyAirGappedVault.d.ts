import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Backup Logically Air Gapped Vault.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.LogicallyAirGappedVault("example", {
 *     name: "lag-example-vault",
 *     maxRetentionDays: 7,
 *     minRetentionDays: 7,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup Logically Air Gapped Vault using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/logicallyAirGappedVault:LogicallyAirGappedVault example lag-example-vault
 * ```
 */
export declare class LogicallyAirGappedVault extends pulumi.CustomResource {
    /**
     * Get an existing LogicallyAirGappedVault resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogicallyAirGappedVaultState, opts?: pulumi.CustomResourceOptions): LogicallyAirGappedVault;
    /**
     * Returns true if the given object is an instance of LogicallyAirGappedVault.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogicallyAirGappedVault;
    /**
     * The ARN of the Logically Air Gapped Backup Vault.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Maximum retention period that the Logically Air Gapped Backup Vault retains recovery points.
     */
    readonly maxRetentionDays: pulumi.Output<number>;
    /**
     * Minimum retention period that the Logically Air Gapped Backup Vault retains recovery points.
     */
    readonly minRetentionDays: pulumi.Output<number>;
    /**
     * Name of the Logically Air Gapped Backup Vault to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.backup.LogicallyAirGappedVaultTimeouts | undefined>;
    /**
     * Create a LogicallyAirGappedVault resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogicallyAirGappedVaultArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogicallyAirGappedVault resources.
 */
export interface LogicallyAirGappedVaultState {
    /**
     * The ARN of the Logically Air Gapped Backup Vault.
     */
    arn?: pulumi.Input<string>;
    /**
     * Maximum retention period that the Logically Air Gapped Backup Vault retains recovery points.
     */
    maxRetentionDays?: pulumi.Input<number>;
    /**
     * Minimum retention period that the Logically Air Gapped Backup Vault retains recovery points.
     */
    minRetentionDays?: pulumi.Input<number>;
    /**
     * Name of the Logically Air Gapped Backup Vault to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.backup.LogicallyAirGappedVaultTimeouts>;
}
/**
 * The set of arguments for constructing a LogicallyAirGappedVault resource.
 */
export interface LogicallyAirGappedVaultArgs {
    /**
     * Maximum retention period that the Logically Air Gapped Backup Vault retains recovery points.
     */
    maxRetentionDays: pulumi.Input<number>;
    /**
     * Minimum retention period that the Logically Air Gapped Backup Vault retains recovery points.
     */
    minRetentionDays: pulumi.Input<number>;
    /**
     * Name of the Logically Air Gapped Backup Vault to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.backup.LogicallyAirGappedVaultTimeouts>;
}
