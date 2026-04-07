import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Redshift Snapshot Copy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.SnapshotCopy("example", {
 *     clusterIdentifier: exampleAwsRedshiftCluster.id,
 *     destinationRegion: "us-east-1",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Snapshot Copy using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/snapshotCopy:SnapshotCopy example cluster-id-12345678
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
     * Identifier of the source cluster.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * AWS Region to copy snapshots to.
     *
     * The following arguments are optional:
     */
    readonly destinationRegion: pulumi.Output<string>;
    /**
     * Number of days to retain newly copied snapshots in the destination AWS Region after they are copied from the source AWS Region. If the value is `-1`, the manual snapshot is retained indefinitely.
     */
    readonly manualSnapshotRetentionPeriod: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Number of days to retain automated snapshots in the destination region after they are copied from the source region.
     */
    readonly retentionPeriod: pulumi.Output<number>;
    /**
     * Name of the snapshot copy grant to use when snapshots of an AWS KMS-encrypted cluster are copied to the destination region.
     */
    readonly snapshotCopyGrantName: pulumi.Output<string | undefined>;
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
     * Identifier of the source cluster.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * AWS Region to copy snapshots to.
     *
     * The following arguments are optional:
     */
    destinationRegion?: pulumi.Input<string>;
    /**
     * Number of days to retain newly copied snapshots in the destination AWS Region after they are copied from the source AWS Region. If the value is `-1`, the manual snapshot is retained indefinitely.
     */
    manualSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Number of days to retain automated snapshots in the destination region after they are copied from the source region.
     */
    retentionPeriod?: pulumi.Input<number>;
    /**
     * Name of the snapshot copy grant to use when snapshots of an AWS KMS-encrypted cluster are copied to the destination region.
     */
    snapshotCopyGrantName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SnapshotCopy resource.
 */
export interface SnapshotCopyArgs {
    /**
     * Identifier of the source cluster.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * AWS Region to copy snapshots to.
     *
     * The following arguments are optional:
     */
    destinationRegion: pulumi.Input<string>;
    /**
     * Number of days to retain newly copied snapshots in the destination AWS Region after they are copied from the source AWS Region. If the value is `-1`, the manual snapshot is retained indefinitely.
     */
    manualSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Number of days to retain automated snapshots in the destination region after they are copied from the source region.
     */
    retentionPeriod?: pulumi.Input<number>;
    /**
     * Name of the snapshot copy grant to use when snapshots of an AWS KMS-encrypted cluster are copied to the destination region.
     */
    snapshotCopyGrantName?: pulumi.Input<string>;
}
