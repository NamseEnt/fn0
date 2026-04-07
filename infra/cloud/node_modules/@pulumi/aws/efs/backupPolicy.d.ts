import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic File System (EFS) Backup Policy resource.
 * Backup policies turn automatic backups on or off for an existing file system.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const fs = new aws.efs.FileSystem("fs", {creationToken: "my-product"});
 * const policy = new aws.efs.BackupPolicy("policy", {
 *     fileSystemId: fs.id,
 *     backupPolicy: {
 *         status: "ENABLED",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the EFS backup policies using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:efs/backupPolicy:BackupPolicy example fs-6fa144c6
 * ```
 */
export declare class BackupPolicy extends pulumi.CustomResource {
    /**
     * Get an existing BackupPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BackupPolicyState, opts?: pulumi.CustomResourceOptions): BackupPolicy;
    /**
     * Returns true if the given object is an instance of BackupPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BackupPolicy;
    /**
     * A backupPolicy object (documented below).
     */
    readonly backupPolicy: pulumi.Output<outputs.efs.BackupPolicyBackupPolicy>;
    /**
     * The ID of the EFS file system.
     */
    readonly fileSystemId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a BackupPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BackupPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BackupPolicy resources.
 */
export interface BackupPolicyState {
    /**
     * A backupPolicy object (documented below).
     */
    backupPolicy?: pulumi.Input<inputs.efs.BackupPolicyBackupPolicy>;
    /**
     * The ID of the EFS file system.
     */
    fileSystemId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BackupPolicy resource.
 */
export interface BackupPolicyArgs {
    /**
     * A backupPolicy object (documented below).
     */
    backupPolicy: pulumi.Input<inputs.efs.BackupPolicyBackupPolicy>;
    /**
     * The ID of the EFS file system.
     */
    fileSystemId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
