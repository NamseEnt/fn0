import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a Snapshot of a snapshot.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ebs.Volume("example", {
 *     availabilityZone: "us-west-2a",
 *     size: 40,
 *     tags: {
 *         Name: "HelloWorld",
 *     },
 * });
 * const exampleSnapshot = new aws.ebs.Snapshot("example_snapshot", {
 *     volumeId: example.id,
 *     tags: {
 *         Name: "HelloWorld_snap",
 *     },
 * });
 * const exampleCopy = new aws.ebs.SnapshotCopy("example_copy", {
 *     sourceSnapshotId: exampleSnapshot.id,
 *     sourceRegion: "us-west-2",
 *     tags: {
 *         Name: "HelloWorld_copy_snap",
 *     },
 * });
 * ```
 */
export declare class SnapshotCopy extends pulumi.CustomResource {
    /**
     * Get an existing SnapshotCopy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SnapshotCopyState, opts?: pulumi.CustomResourceOptions): SnapshotCopy;
    /**
     * Returns true if the given object is an instance of SnapshotCopy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SnapshotCopy;
    /**
     * Amazon Resource Name (ARN) of the EBS Snapshot.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies a completion duration to initiate a time-based snapshot copy. Time-based snapshot copy operations complete within the specified duration.  Value must be between 15 and 2880 minutes, in 15 minute increments only.
     */
    readonly completionDurationMinutes: pulumi.Output<number | undefined>;
    /**
     * The data encryption key identifier for the snapshot.
     */
    readonly dataEncryptionKeyId: pulumi.Output<string>;
    /**
     * A description of what the snapshot is.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Whether the snapshot is encrypted.
     */
    readonly encrypted: pulumi.Output<boolean | undefined>;
    /**
     * The ARN for the KMS encryption key.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    readonly outpostArn: pulumi.Output<string>;
    /**
     * Value from an Amazon-maintained list (`amazon`, `aws-marketplace`, `microsoft`) of snapshot owners.
     */
    readonly ownerAlias: pulumi.Output<string>;
    /**
     * The AWS account ID of the snapshot owner.
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
     * The region of the source snapshot.
     */
    readonly sourceRegion: pulumi.Output<string>;
    /**
     * The ARN for the snapshot to be copied.
     */
    readonly sourceSnapshotId: pulumi.Output<string>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    readonly storageTier: pulumi.Output<string>;
    /**
     * A map of tags for the snapshot.
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
     * Create a SnapshotCopy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SnapshotCopyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SnapshotCopy resources.
 */
export interface SnapshotCopyState {
    /**
     * Amazon Resource Name (ARN) of the EBS Snapshot.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies a completion duration to initiate a time-based snapshot copy. Time-based snapshot copy operations complete within the specified duration.  Value must be between 15 and 2880 minutes, in 15 minute increments only.
     */
    completionDurationMinutes?: pulumi.Input<number>;
    /**
     * The data encryption key identifier for the snapshot.
     */
    dataEncryptionKeyId?: pulumi.Input<string>;
    /**
     * A description of what the snapshot is.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether the snapshot is encrypted.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * The ARN for the KMS encryption key.
     */
    kmsKeyId?: pulumi.Input<string>;
    outpostArn?: pulumi.Input<string>;
    /**
     * Value from an Amazon-maintained list (`amazon`, `aws-marketplace`, `microsoft`) of snapshot owners.
     */
    ownerAlias?: pulumi.Input<string>;
    /**
     * The AWS account ID of the snapshot owner.
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
     * The region of the source snapshot.
     */
    sourceRegion?: pulumi.Input<string>;
    /**
     * The ARN for the snapshot to be copied.
     */
    sourceSnapshotId?: pulumi.Input<string>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    storageTier?: pulumi.Input<string>;
    /**
     * A map of tags for the snapshot.
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
 * The set of arguments for constructing a SnapshotCopy resource.
 */
export interface SnapshotCopyArgs {
    /**
     * Specifies a completion duration to initiate a time-based snapshot copy. Time-based snapshot copy operations complete within the specified duration.  Value must be between 15 and 2880 minutes, in 15 minute increments only.
     */
    completionDurationMinutes?: pulumi.Input<number>;
    /**
     * A description of what the snapshot is.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether the snapshot is encrypted.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * The ARN for the KMS encryption key.
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
     * The region of the source snapshot.
     */
    sourceRegion: pulumi.Input<string>;
    /**
     * The ARN for the snapshot to be copied.
     */
    sourceSnapshotId: pulumi.Input<string>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    storageTier?: pulumi.Input<string>;
    /**
     * A map of tags for the snapshot.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the number of days for which to temporarily restore an archived snapshot. Required for temporary restores only. The snapshot will be automatically re-archived after this period.
     */
    temporaryRestoreDays?: pulumi.Input<number>;
}
