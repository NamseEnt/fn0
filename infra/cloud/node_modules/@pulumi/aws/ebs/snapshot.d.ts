import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a Snapshot of an EBS Volume.
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
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EBS Snapshot using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ebs/snapshot:Snapshot id snap-049df61146c4d7901
 * ```
 */
export declare class Snapshot extends pulumi.CustomResource {
    /**
     * Get an existing Snapshot resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SnapshotState, opts?: pulumi.CustomResourceOptions): Snapshot;
    /**
     * Returns true if the given object is an instance of Snapshot.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Snapshot;
    /**
     * Amazon Resource Name (ARN) of the EBS Snapshot.
     */
    readonly arn: pulumi.Output<string>;
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
    readonly encrypted: pulumi.Output<boolean>;
    /**
     * The ARN for the KMS encryption key.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Outpost on which to create a local snapshot.
     */
    readonly outpostArn: pulumi.Output<string | undefined>;
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
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    readonly storageTier: pulumi.Output<string>;
    /**
     * A map of tags to assign to the snapshot. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The Volume ID of which to make a snapshot.
     */
    readonly volumeId: pulumi.Output<string>;
    /**
     * The size of the drive in GiBs.
     */
    readonly volumeSize: pulumi.Output<number>;
    /**
     * Create a Snapshot resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SnapshotArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Snapshot resources.
 */
export interface SnapshotState {
    /**
     * Amazon Resource Name (ARN) of the EBS Snapshot.
     */
    arn?: pulumi.Input<string>;
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
    /**
     * The Amazon Resource Name (ARN) of the Outpost on which to create a local snapshot.
     */
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
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    storageTier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the snapshot. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The Volume ID of which to make a snapshot.
     */
    volumeId?: pulumi.Input<string>;
    /**
     * The size of the drive in GiBs.
     */
    volumeSize?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Snapshot resource.
 */
export interface SnapshotArgs {
    /**
     * A description of what the snapshot is.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Outpost on which to create a local snapshot.
     */
    outpostArn?: pulumi.Input<string>;
    /**
     * Indicates whether to permanently restore an archived snapshot.
     */
    permanentRestore?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the storage tier. Valid values are `archive` and `standard`. Default value is `standard`.
     */
    storageTier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the snapshot. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the number of days for which to temporarily restore an archived snapshot. Required for temporary restores only. The snapshot will be automatically re-archived after this period.
     */
    temporaryRestoreDays?: pulumi.Input<number>;
    /**
     * The Volume ID of which to make a snapshot.
     */
    volumeId: pulumi.Input<string>;
}
