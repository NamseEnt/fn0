import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ### Testing Glacier Vault Lock Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleVault = new aws.glacier.Vault("example", {name: "example"});
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         actions: ["glacier:DeleteArchive"],
 *         effect: "Deny",
 *         resources: [exampleVault.arn],
 *         conditions: [{
 *             test: "NumericLessThanEquals",
 *             variable: "glacier:ArchiveAgeinDays",
 *             values: ["365"],
 *         }],
 *     }],
 * });
 * const exampleVaultLock = new aws.glacier.VaultLock("example", {
 *     completeLock: false,
 *     policy: example.apply(example => example.json),
 *     vaultName: exampleVault.name,
 * });
 * ```
 *
 * ### Permanently Applying Glacier Vault Lock Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glacier.VaultLock("example", {
 *     completeLock: true,
 *     policy: exampleAwsIamPolicyDocument.json,
 *     vaultName: exampleAwsGlacierVault.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glacier Vault Locks using the Glacier Vault name. For example:
 *
 * ```sh
 * $ pulumi import aws:glacier/vaultLock:VaultLock example example-vault
 * ```
 */
export declare class VaultLock extends pulumi.CustomResource {
    /**
     * Get an existing VaultLock resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VaultLockState, opts?: pulumi.CustomResourceOptions): VaultLock;
    /**
     * Returns true if the given object is an instance of VaultLock.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VaultLock;
    /**
     * Boolean whether to permanently apply this Glacier Lock Policy. Once completed, this cannot be undone. If set to `false`, the Glacier Lock Policy remains in a testing mode for 24 hours. After that time, the Glacier Lock Policy is automatically removed by Glacier and the this provider resource will show as needing recreation. Changing this from `false` to `true` will show as resource recreation, which is expected. Changing this from `true` to `false` is not possible unless the Glacier Vault is recreated at the same time.
     */
    readonly completeLock: pulumi.Output<boolean>;
    /**
     * Allow this provider to ignore the error returned when attempting to delete the Glacier Lock Policy. This can be used to delete or recreate the Glacier Vault via this provider, for example, if the Glacier Vault Lock policy permits that action. This should only be used in conjunction with `completeLock` being set to `true`.
     */
    readonly ignoreDeletionError: pulumi.Output<boolean | undefined>;
    /**
     * JSON string containing the IAM policy to apply as the Glacier Vault Lock policy.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the Glacier Vault.
     */
    readonly vaultName: pulumi.Output<string>;
    /**
     * Create a VaultLock resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VaultLockArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VaultLock resources.
 */
export interface VaultLockState {
    /**
     * Boolean whether to permanently apply this Glacier Lock Policy. Once completed, this cannot be undone. If set to `false`, the Glacier Lock Policy remains in a testing mode for 24 hours. After that time, the Glacier Lock Policy is automatically removed by Glacier and the this provider resource will show as needing recreation. Changing this from `false` to `true` will show as resource recreation, which is expected. Changing this from `true` to `false` is not possible unless the Glacier Vault is recreated at the same time.
     */
    completeLock?: pulumi.Input<boolean>;
    /**
     * Allow this provider to ignore the error returned when attempting to delete the Glacier Lock Policy. This can be used to delete or recreate the Glacier Vault via this provider, for example, if the Glacier Vault Lock policy permits that action. This should only be used in conjunction with `completeLock` being set to `true`.
     */
    ignoreDeletionError?: pulumi.Input<boolean>;
    /**
     * JSON string containing the IAM policy to apply as the Glacier Vault Lock policy.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the Glacier Vault.
     */
    vaultName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VaultLock resource.
 */
export interface VaultLockArgs {
    /**
     * Boolean whether to permanently apply this Glacier Lock Policy. Once completed, this cannot be undone. If set to `false`, the Glacier Lock Policy remains in a testing mode for 24 hours. After that time, the Glacier Lock Policy is automatically removed by Glacier and the this provider resource will show as needing recreation. Changing this from `false` to `true` will show as resource recreation, which is expected. Changing this from `true` to `false` is not possible unless the Glacier Vault is recreated at the same time.
     */
    completeLock: pulumi.Input<boolean>;
    /**
     * Allow this provider to ignore the error returned when attempting to delete the Glacier Lock Policy. This can be used to delete or recreate the Glacier Vault via this provider, for example, if the Glacier Vault Lock policy permits that action. This should only be used in conjunction with `completeLock` being set to `true`.
     */
    ignoreDeletionError?: pulumi.Input<boolean>;
    /**
     * JSON string containing the IAM policy to apply as the Glacier Vault Lock policy.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the Glacier Vault.
     */
    vaultName: pulumi.Input<string>;
}
