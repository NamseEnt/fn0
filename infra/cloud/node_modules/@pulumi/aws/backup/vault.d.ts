import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Backup vault resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.Vault("example", {
 *     name: "example_backup_vault",
 *     kmsKeyArn: exampleAwsKmsKey.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup vault using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/vault:Vault test-vault TestVault
 * ```
 */
export declare class Vault extends pulumi.CustomResource {
    /**
     * Get an existing Vault resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VaultState, opts?: pulumi.CustomResourceOptions): Vault;
    /**
     * Returns true if the given object is an instance of Vault.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Vault;
    /**
     * The ARN of the vault.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A boolean that indicates that all recovery points stored in the vault are deleted so that the vault can be destroyed without error.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * The server-side encryption key that is used to protect your backups.
     */
    readonly kmsKeyArn: pulumi.Output<string>;
    /**
     * Name of the backup vault to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The number of recovery points that are stored in a backup vault.
     */
    readonly recoveryPoints: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Vault resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: VaultArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Vault resources.
 */
export interface VaultState {
    /**
     * The ARN of the vault.
     */
    arn?: pulumi.Input<string>;
    /**
     * A boolean that indicates that all recovery points stored in the vault are deleted so that the vault can be destroyed without error.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The server-side encryption key that is used to protect your backups.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the backup vault to create.
     */
    name?: pulumi.Input<string>;
    /**
     * The number of recovery points that are stored in a backup vault.
     */
    recoveryPoints?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a Vault resource.
 */
export interface VaultArgs {
    /**
     * A boolean that indicates that all recovery points stored in the vault are deleted so that the vault can be destroyed without error.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The server-side encryption key that is used to protect your backups.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the backup vault to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
