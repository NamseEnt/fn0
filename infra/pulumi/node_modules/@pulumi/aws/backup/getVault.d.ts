import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get information on an existing backup vault.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.backup.getVault({
 *     name: "example_backup_vault",
 * });
 * ```
 */
export declare function getVault(args: GetVaultArgs, opts?: pulumi.InvokeOptions): Promise<GetVaultResult>;
/**
 * A collection of arguments for invoking getVault.
 */
export interface GetVaultArgs {
    /**
     * Name of the backup vault.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Metadata that you can assign to help organize the resources that you create.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getVault.
 */
export interface GetVaultResult {
    /**
     * ARN of the vault.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Server-side encryption key that is used to protect your backups.
     */
    readonly kmsKeyArn: string;
    readonly name: string;
    /**
     * Number of recovery points that are stored in a backup vault.
     */
    readonly recoveryPoints: number;
    readonly region: string;
    /**
     * Metadata that you can assign to help organize the resources that you create.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Use this data source to get information on an existing backup vault.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.backup.getVault({
 *     name: "example_backup_vault",
 * });
 * ```
 */
export declare function getVaultOutput(args: GetVaultOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVaultResult>;
/**
 * A collection of arguments for invoking getVault.
 */
export interface GetVaultOutputArgs {
    /**
     * Name of the backup vault.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the resources that you create.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
