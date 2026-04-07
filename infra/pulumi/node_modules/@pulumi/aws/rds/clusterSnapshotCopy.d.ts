import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an RDS database cluster snapshot copy. For managing RDS database instance snapshot copies, see the `aws.rds.SnapshotCopy` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rds.Cluster("example", {
 *     clusterIdentifier: "aurora-cluster-demo",
 *     databaseName: "test",
 *     engine: aws.rds.EngineType.AuroraMysql,
 *     masterUsername: "tfacctest",
 *     masterPassword: "avoid-plaintext-passwords",
 *     skipFinalSnapshot: true,
 * });
 * const exampleClusterSnapshot = new aws.rds.ClusterSnapshot("example", {
 *     dbClusterIdentifier: example.clusterIdentifier,
 *     dbClusterSnapshotIdentifier: "example",
 * });
 * const exampleClusterSnapshotCopy = new aws.rds.ClusterSnapshotCopy("example", {
 *     sourceDbClusterSnapshotIdentifier: exampleClusterSnapshot.dbClusterSnapshotArn,
 *     targetDbClusterSnapshotIdentifier: "example-copy",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_rds_cluster_snapshot_copy` using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/clusterSnapshotCopy:ClusterSnapshotCopy example my-snapshot
 * ```
 */
export declare class ClusterSnapshotCopy extends pulumi.CustomResource {
    /**
     * Get an existing ClusterSnapshotCopy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterSnapshotCopyState, opts?: pulumi.CustomResourceOptions): ClusterSnapshotCopy;
    /**
     * Returns true if the given object is an instance of ClusterSnapshotCopy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterSnapshotCopy;
    /**
     * Specifies the allocated storage size in gigabytes (GB).
     */
    readonly allocatedStorage: pulumi.Output<number>;
    /**
     * Whether to copy existing tags. Defaults to `false`.
     */
    readonly copyTags: pulumi.Output<boolean | undefined>;
    /**
     * The Amazon Resource Name (ARN) for the DB cluster snapshot.
     */
    readonly dbClusterSnapshotArn: pulumi.Output<string>;
    /**
     * The Destination region to place snapshot copy.
     */
    readonly destinationRegion: pulumi.Output<string | undefined>;
    /**
     * Specifies the name of the database engine.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * Specifies the version of the database engine.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * KMS key ID.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * License model information for the restored DB instance.
     */
    readonly licenseModel: pulumi.Output<string>;
    /**
     * URL that contains a Signature Version 4 signed request.
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
     * Identifier of the source snapshot.
     */
    readonly sourceDbClusterSnapshotIdentifier: pulumi.Output<string>;
    /**
     * Specifies whether the DB cluster snapshot is encrypted.
     */
    readonly storageEncrypted: pulumi.Output<boolean>;
    /**
     * Specifies the storage type associated with DB cluster snapshot.
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
     * Identifier for the snapshot.
     *
     * The following arguments are optional:
     */
    readonly targetDbClusterSnapshotIdentifier: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.rds.ClusterSnapshotCopyTimeouts | undefined>;
    /**
     * Provides the VPC ID associated with the DB cluster snapshot.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a ClusterSnapshotCopy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterSnapshotCopyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterSnapshotCopy resources.
 */
export interface ClusterSnapshotCopyState {
    /**
     * Specifies the allocated storage size in gigabytes (GB).
     */
    allocatedStorage?: pulumi.Input<number>;
    /**
     * Whether to copy existing tags. Defaults to `false`.
     */
    copyTags?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) for the DB cluster snapshot.
     */
    dbClusterSnapshotArn?: pulumi.Input<string>;
    /**
     * The Destination region to place snapshot copy.
     */
    destinationRegion?: pulumi.Input<string>;
    /**
     * Specifies the name of the database engine.
     */
    engine?: pulumi.Input<string>;
    /**
     * Specifies the version of the database engine.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * KMS key ID.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * License model information for the restored DB instance.
     */
    licenseModel?: pulumi.Input<string>;
    /**
     * URL that contains a Signature Version 4 signed request.
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
     * Identifier of the source snapshot.
     */
    sourceDbClusterSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * Specifies whether the DB cluster snapshot is encrypted.
     */
    storageEncrypted?: pulumi.Input<boolean>;
    /**
     * Specifies the storage type associated with DB cluster snapshot.
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
     * Identifier for the snapshot.
     *
     * The following arguments are optional:
     */
    targetDbClusterSnapshotIdentifier?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.rds.ClusterSnapshotCopyTimeouts>;
    /**
     * Provides the VPC ID associated with the DB cluster snapshot.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ClusterSnapshotCopy resource.
 */
export interface ClusterSnapshotCopyArgs {
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
     * URL that contains a Signature Version 4 signed request.
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
     * Identifier of the source snapshot.
     */
    sourceDbClusterSnapshotIdentifier: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Identifier for the snapshot.
     *
     * The following arguments are optional:
     */
    targetDbClusterSnapshotIdentifier: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.rds.ClusterSnapshotCopyTimeouts>;
}
