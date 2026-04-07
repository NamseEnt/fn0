import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Backup vault policy resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const exampleVault = new aws.backup.Vault("example", {name: "example"});
 * const example = pulumi.all([current, exampleVault.arn]).apply(([current, arn]) => aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: [current.accountId],
 *         }],
 *         actions: [
 *             "backup:DescribeBackupVault",
 *             "backup:DeleteBackupVault",
 *             "backup:PutBackupVaultAccessPolicy",
 *             "backup:DeleteBackupVaultAccessPolicy",
 *             "backup:GetBackupVaultAccessPolicy",
 *             "backup:StartBackupJob",
 *             "backup:GetBackupVaultNotifications",
 *             "backup:PutBackupVaultNotifications",
 *         ],
 *         resources: [arn],
 *     }],
 * }));
 * const exampleVaultPolicy = new aws.backup.VaultPolicy("example", {
 *     backupVaultName: exampleVault.name,
 *     policy: example.apply(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup vault policy using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/vaultPolicy:VaultPolicy test TestVault
 * ```
 */
export declare class VaultPolicy extends pulumi.CustomResource {
    /**
     * Get an existing VaultPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VaultPolicyState, opts?: pulumi.CustomResourceOptions): VaultPolicy;
    /**
     * Returns true if the given object is an instance of VaultPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VaultPolicy;
    /**
     * The ARN of the vault.
     */
    readonly backupVaultArn: pulumi.Output<string>;
    /**
     * Name of the backup vault to add policy for.
     */
    readonly backupVaultName: pulumi.Output<string>;
    /**
     * The backup vault access policy document in JSON format.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a VaultPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VaultPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VaultPolicy resources.
 */
export interface VaultPolicyState {
    /**
     * The ARN of the vault.
     */
    backupVaultArn?: pulumi.Input<string>;
    /**
     * Name of the backup vault to add policy for.
     */
    backupVaultName?: pulumi.Input<string>;
    /**
     * The backup vault access policy document in JSON format.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VaultPolicy resource.
 */
export interface VaultPolicyArgs {
    /**
     * Name of the backup vault to add policy for.
     */
    backupVaultName: pulumi.Input<string>;
    /**
     * The backup vault access policy document in JSON format.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
