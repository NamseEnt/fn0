import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Imports a disk image from S3 as a Snapshot.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ebs.SnapshotImport("example", {
 *     diskContainer: {
 *         format: "VHD",
 *         userBucket: {
 *             s3Bucket: "disk-images",
 *             s3Key: "source.vhd",
 *         },
 *     },
 *     roleName: "disk-image-import",
 *     tags: {
 *         Name: "HelloWorld",
 *     },
 * });
 * ```
 */
export declare class SnapshotImport extends pulumi.CustomResource {
    /**
     * Get an existing SnapshotImport resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SnapshotImportState, opts?: pulumi.CustomResourceOptions): SnapshotImport;
    /**
     * Returns true if the given object is an instance of SnapshotImport.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SnapshotImport;
    /**
     * Amazon Resource Name (ARN) of the EBS Snapshot.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The client-specific data. Detailed below.
     */
    readonly clientData: pulumi.Output<outputs.ebs.SnapshotImportClientData | undefined>;
    /**
     * The data encryption key identifier for the snapshot.
     */
    readonly dataEncryptionKeyId: pulumi.Output<string>;
    /**
     * The description string for the import snapshot task.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Information about the disk container. Detailed below.
     */
    readonly diskContainer: pulumi.Output<outputs.ebs.SnapshotImportDiskContainer>;
    /**
     * Specifies whether the destination snapshot of the imported image should be encrypted. The default KMS key for EBS is used unless you specify a non-default KMS key using KmsKeyId.
     */
    readonly encrypted: pulumi.Output<boolean | undefined>;
    /**
     * An identifier for the symmetric KMS key to use when creating the encrypted snapshot. This parameter is only required if you want to use a non-default KMS key; if this parameter is not specified, the default KMS key for EBS is used. If a KmsKeyId is specified, the Encrypted flag must also be set.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    readonly outpostArn: pulumi.Output<string>;
    /**
     * Value from an Amazon-maintained list (`amazon`, `aws-marketplace`, `microsoft`) of snapshot owners.
     */
    readonly ownerAlias: pulumi.Output<string>;
    /**
     * The AWS account ID of the EBS snapshot owner.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Indicates whether to permanently restore an archived snapshot.
     */
    readonly permanentRestore: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the IAM Role the VM Import/Export service will assume. This role needs certain permissions. See https://docs.aws.amazon.com/vm-import/latest/userguide/vmie_prereqs.html#vmimport-role. Default: `vmimport`
     */
    readonly roleName: pulumi.Output<string | undefined>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    readonly storageTier: pulumi.Output<string>;
    /**
     * A map of tags to assign to the snapshot.
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
     * Specifies the number of days for which to temporarily restore an archived snapshot. Required for temporary restores only. The snapshot will be automatically re-archived after this period.
     */
    readonly temporaryRestoreDays: pulumi.Output<number | undefined>;
    readonly volumeId: pulumi.Output<string>;
    /**
     * The size of the drive in GiBs.
     */
    readonly volumeSize: pulumi.Output<number>;
    /**
     * Create a SnapshotImport resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SnapshotImportArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SnapshotImport resources.
 */
export interface SnapshotImportState {
    /**
     * Amazon Resource Name (ARN) of the EBS Snapshot.
     */
    arn?: pulumi.Input<string>;
    /**
     * The client-specific data. Detailed below.
     */
    clientData?: pulumi.Input<inputs.ebs.SnapshotImportClientData>;
    /**
     * The data encryption key identifier for the snapshot.
     */
    dataEncryptionKeyId?: pulumi.Input<string>;
    /**
     * The description string for the import snapshot task.
     */
    description?: pulumi.Input<string>;
    /**
     * Information about the disk container. Detailed below.
     */
    diskContainer?: pulumi.Input<inputs.ebs.SnapshotImportDiskContainer>;
    /**
     * Specifies whether the destination snapshot of the imported image should be encrypted. The default KMS key for EBS is used unless you specify a non-default KMS key using KmsKeyId.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * An identifier for the symmetric KMS key to use when creating the encrypted snapshot. This parameter is only required if you want to use a non-default KMS key; if this parameter is not specified, the default KMS key for EBS is used. If a KmsKeyId is specified, the Encrypted flag must also be set.
     */
    kmsKeyId?: pulumi.Input<string>;
    outpostArn?: pulumi.Input<string>;
    /**
     * Value from an Amazon-maintained list (`amazon`, `aws-marketplace`, `microsoft`) of snapshot owners.
     */
    ownerAlias?: pulumi.Input<string>;
    /**
     * The AWS account ID of the EBS snapshot owner.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Indicates whether to permanently restore an archived snapshot.
     */
    permanentRestore?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the IAM Role the VM Import/Export service will assume. This role needs certain permissions. See https://docs.aws.amazon.com/vm-import/latest/userguide/vmie_prereqs.html#vmimport-role. Default: `vmimport`
     */
    roleName?: pulumi.Input<string>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    storageTier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the snapshot.
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
     * Specifies the number of days for which to temporarily restore an archived snapshot. Required for temporary restores only. The snapshot will be automatically re-archived after this period.
     */
    temporaryRestoreDays?: pulumi.Input<number>;
    volumeId?: pulumi.Input<string>;
    /**
     * The size of the drive in GiBs.
     */
    volumeSize?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a SnapshotImport resource.
 */
export interface SnapshotImportArgs {
    /**
     * The client-specific data. Detailed below.
     */
    clientData?: pulumi.Input<inputs.ebs.SnapshotImportClientData>;
    /**
     * The description string for the import snapshot task.
     */
    description?: pulumi.Input<string>;
    /**
     * Information about the disk container. Detailed below.
     */
    diskContainer: pulumi.Input<inputs.ebs.SnapshotImportDiskContainer>;
    /**
     * Specifies whether the destination snapshot of the imported image should be encrypted. The default KMS key for EBS is used unless you specify a non-default KMS key using KmsKeyId.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * An identifier for the symmetric KMS key to use when creating the encrypted snapshot. This parameter is only required if you want to use a non-default KMS key; if this parameter is not specified, the default KMS key for EBS is used. If a KmsKeyId is specified, the Encrypted flag must also be set.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Indicates whether to permanently restore an archived snapshot.
     */
    permanentRestore?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the IAM Role the VM Import/Export service will assume. This role needs certain permissions. See https://docs.aws.amazon.com/vm-import/latest/userguide/vmie_prereqs.html#vmimport-role. Default: `vmimport`
     */
    roleName?: pulumi.Input<string>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    storageTier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the snapshot.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the number of days for which to temporarily restore an archived snapshot. Required for temporary restores only. The snapshot will be automatically re-archived after this period.
     */
    temporaryRestoreDays?: pulumi.Input<number>;
}
