import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a MemoryDB Snapshot.
 *
 * More information about snapshot and restore can be found in the [MemoryDB User Guide](https://docs.aws.amazon.com/memorydb/latest/devguide/snapshots.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.memorydb.Snapshot("example", {
 *     clusterName: exampleAwsMemorydbCluster.name,
 *     name: "my-snapshot",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a snapshot using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:memorydb/snapshot:Snapshot example my-snapshot
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
     * The ARN of the snapshot.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The configuration of the cluster from which the snapshot was taken.
     */
    readonly clusterConfigurations: pulumi.Output<outputs.memorydb.SnapshotClusterConfiguration[]>;
    /**
     * Name of the MemoryDB cluster to take a snapshot of.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * ARN of the KMS key used to encrypt the snapshot at rest.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Name of the snapshot. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Indicates whether the snapshot is from an automatic backup (`automated`) or was created manually (`manual`).
     */
    readonly source: pulumi.Output<string>;
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
     * The ARN of the snapshot.
     */
    arn?: pulumi.Input<string>;
    /**
     * The configuration of the cluster from which the snapshot was taken.
     */
    clusterConfigurations?: pulumi.Input<pulumi.Input<inputs.memorydb.SnapshotClusterConfiguration>[]>;
    /**
     * Name of the MemoryDB cluster to take a snapshot of.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the snapshot at rest.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the snapshot. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Indicates whether the snapshot is from an automatic backup (`automated`) or was created manually (`manual`).
     */
    source?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Snapshot resource.
 */
export interface SnapshotArgs {
    /**
     * Name of the MemoryDB cluster to take a snapshot of.
     */
    clusterName: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the snapshot at rest.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the snapshot. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
