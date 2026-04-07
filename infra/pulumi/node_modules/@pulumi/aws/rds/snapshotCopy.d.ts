import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an RDS database instance snapshot copy. For managing RDS database cluster snapshots, see the `aws.rds.ClusterSnapshot` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rds.Instance("example", {
 *     allocatedStorage: 10,
 *     engine: "mysql",
 *     engineVersion: "5.6.21",
 *     instanceClass: aws.rds.InstanceType.T2_Micro,
 *     dbName: "baz",
 *     password: "barbarbarbar",
 *     username: "foo",
 *     maintenanceWindow: "Fri:09:00-Fri:09:30",
 *     backupRetentionPeriod: 0,
 *     parameterGroupName: "default.mysql5.6",
 * });
 * const exampleSnapshot = new aws.rds.Snapshot("example", {
 *     dbInstanceIdentifier: example.identifier,
 *     dbSnapshotIdentifier: "testsnapshot1234",
 * });
 * const exampleSnapshotCopy = new aws.rds.SnapshotCopy("example", {
 *     sourceDbSnapshotIdentifier: exampleSnapshot.dbSnapshotArn,
 *     targetDbSnapshotIdentifier: "testsnapshot1234-copy",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_db_snapshot_copy` using the snapshot identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/snapshotCopy:SnapshotCopy example my-snapshot
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
     * Specifies the allocated storage size in gigabytes (GB).
     */
    readonly allocatedStorage: pulumi.Output<number>;
    /**
     * Specifies the name of the Availability Zone the DB instance was located in at the time of the DB snapshot.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Whether to copy existing tags. Defaults to `false`.
     */
    readonly copyTags: pulumi.Output<boolean | undefined>;
    /**
     * The Amazon Resource Name (ARN) for the DB snapshot.
     */
    readonly dbSnapshotArn: pulumi.Output<string>;
    /**
     * The Destination region to place snapshot copy.
     */
    readonly destinationRegion: pulumi.Output<string | undefined>;
    /**
     * Specifies whether the DB snapshot is encrypted.
     */
    readonly encrypted: pulumi.Output<boolean>;
    /**
     * Specifies the name of the database engine.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * Specifies the version of the database engine.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Specifies the Provisioned IOPS (I/O operations per second) value of the DB instance at the time of the snapshot.
     */
    readonly iops: pulumi.Output<number>;
    /**
     * KMS key ID.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * License model information for the restored DB instance.
     */
    readonly licenseModel: pulumi.Output<string>;
    /**
     * The name of an option group to associate with the copy of the snapshot.
     */
    readonly optionGroupName: pulumi.Output<string>;
    readonly port: pulumi.Output<number>;
    /**
     * he URL that contains a Signature Version 4 signed request.
     */
    readonly presignedUrl: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of AWS Account IDs to share the snapshot with. Use `all` to make the snapshot public.
     */
    readonly sharedAccounts: pulumi.Output<string[] | undefined>;
    readonly snapshotType: pulumi.Output<string>;
    /**
     * Snapshot identifier of the source snapshot.
     */
    readonly sourceDbSnapshotIdentifier: pulumi.Output<string>;
    /**
     * The region that the DB snapshot was created in or copied from.
     */
    readonly sourceRegion: pulumi.Output<string>;
    /**
     * Specifies the storage type associated with DB snapshot.
     */
    readonly storageType: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The external custom Availability Zone.
     */
    readonly targetCustomAvailabilityZone: pulumi.Output<string | undefined>;
    /**
     * The Identifier for the snapshot.
     */
    readonly targetDbSnapshotIdentifier: pulumi.Output<string>;
    /**
     * Provides the VPC ID associated with the DB snapshot.
     */
    readonly vpcId: pulumi.Output<string>;
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
     * Specifies the allocated storage size in gigabytes (GB).
     */
    allocatedStorage?: pulumi.Input<number>;
    /**
     * Specifies the name of the Availability Zone the DB instance was located in at the time of the DB snapshot.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Whether to copy existing tags. Defaults to `false`.
     */
    copyTags?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) for the DB snapshot.
     */
    dbSnapshotArn?: pulumi.Input<string>;
    /**
     * The Destination region to place snapshot copy.
     */
    destinationRegion?: pulumi.Input<string>;
    /**
     * Specifies whether the DB snapshot is encrypted.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * Specifies the name of the database engine.
     */
    engine?: pulumi.Input<string>;
    /**
     * Specifies the version of the database engine.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Specifies the Provisioned IOPS (I/O operations per second) value of the DB instance at the time of the snapshot.
     */
    iops?: pulumi.Input<number>;
    /**
     * KMS key ID.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * License model information for the restored DB instance.
     */
    licenseModel?: pulumi.Input<string>;
    /**
     * The name of an option group to associate with the copy of the snapshot.
     */
    optionGroupName?: pulumi.Input<string>;
    port?: pulumi.Input<number>;
    /**
     * he URL that contains a Signature Version 4 signed request.
     */
    presignedUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of AWS Account IDs to share the snapshot with. Use `all` to make the snapshot public.
     */
    sharedAccounts?: pulumi.Input<pulumi.Input<string>[]>;
    snapshotType?: pulumi.Input<string>;
    /**
     * Snapshot identifier of the source snapshot.
     */
    sourceDbSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * The region that the DB snapshot was created in or copied from.
     */
    sourceRegion?: pulumi.Input<string>;
    /**
     * Specifies the storage type associated with DB snapshot.
     */
    storageType?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The external custom Availability Zone.
     */
    targetCustomAvailabilityZone?: pulumi.Input<string>;
    /**
     * The Identifier for the snapshot.
     */
    targetDbSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * Provides the VPC ID associated with the DB snapshot.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SnapshotCopy resource.
 */
export interface SnapshotCopyArgs {
    /**
     * Whether to copy existing tags. Defaults to `false`.
     */
    copyTags?: pulumi.Input<boolean>;
    /**
     * The Destination region to place snapshot copy.
     */
    destinationRegion?: pulumi.Input<string>;
    /**
     * KMS key ID.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of an option group to associate with the copy of the snapshot.
     */
    optionGroupName?: pulumi.Input<string>;
    /**
     * he URL that contains a Signature Version 4 signed request.
     */
    presignedUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of AWS Account IDs to share the snapshot with. Use `all` to make the snapshot public.
     */
    sharedAccounts?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Snapshot identifier of the source snapshot.
     */
    sourceDbSnapshotIdentifier: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The external custom Availability Zone.
     */
    targetCustomAvailabilityZone?: pulumi.Input<string>;
    /**
     * The Identifier for the snapshot.
     */
    targetDbSnapshotIdentifier: pulumi.Input<string>;
}
