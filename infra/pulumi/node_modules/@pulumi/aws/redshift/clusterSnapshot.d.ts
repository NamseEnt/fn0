import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a Redshift cluster snapshot
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.ClusterSnapshot("example", {
 *     clusterSnapshotName: "example",
 *     clusterSnapshotContent: JSON.stringify({
 *         AllowDBUserOverride: "1",
 *         Client_ID: "ExampleClientID",
 *         App_ID: "example",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Cluster Snapshots using `snapshot_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/clusterSnapshot:ClusterSnapshot test example
 * ```
 */
export declare class ClusterSnapshot extends pulumi.CustomResource {
    /**
     * Get an existing ClusterSnapshot resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterSnapshotState, opts?: pulumi.CustomResourceOptions): ClusterSnapshot;
    /**
     * Returns true if the given object is an instance of ClusterSnapshot.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterSnapshot;
    /**
     * Amazon Resource Name (ARN) of the snapshot.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The cluster identifier for which you want a snapshot.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The Key Management Service (KMS) key ID of the encryption key that was used to encrypt data in the cluster from which the snapshot was taken.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The number of days that a manual snapshot is retained. If the value is `-1`, the manual snapshot is retained indefinitely. Valid values are -1 and between `1` and `3653`.
     */
    readonly manualSnapshotRetentionPeriod: pulumi.Output<number | undefined>;
    /**
     * For manual snapshots, the Amazon Web Services account used to create or copy the snapshot. For automatic snapshots, the owner of the cluster. The owner can perform all snapshot actions, such as sharing a manual snapshot.
     */
    readonly ownerAccount: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A unique identifier for the snapshot that you are requesting. This identifier must be unique for all snapshots within the Amazon Web Services account.
     */
    readonly snapshotIdentifier: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ClusterSnapshot resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterSnapshotArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterSnapshot resources.
 */
export interface ClusterSnapshotState {
    /**
     * Amazon Resource Name (ARN) of the snapshot.
     */
    arn?: pulumi.Input<string>;
    /**
     * The cluster identifier for which you want a snapshot.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The Key Management Service (KMS) key ID of the encryption key that was used to encrypt data in the cluster from which the snapshot was taken.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The number of days that a manual snapshot is retained. If the value is `-1`, the manual snapshot is retained indefinitely. Valid values are -1 and between `1` and `3653`.
     */
    manualSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * For manual snapshots, the Amazon Web Services account used to create or copy the snapshot. For automatic snapshots, the owner of the cluster. The owner can perform all snapshot actions, such as sharing a manual snapshot.
     */
    ownerAccount?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A unique identifier for the snapshot that you are requesting. This identifier must be unique for all snapshots within the Amazon Web Services account.
     */
    snapshotIdentifier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a ClusterSnapshot resource.
 */
export interface ClusterSnapshotArgs {
    /**
     * The cluster identifier for which you want a snapshot.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The number of days that a manual snapshot is retained. If the value is `-1`, the manual snapshot is retained indefinitely. Valid values are -1 and between `1` and `3653`.
     */
    manualSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A unique identifier for the snapshot that you are requesting. This identifier must be unique for all snapshots within the Amazon Web Services account.
     */
    snapshotIdentifier: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
