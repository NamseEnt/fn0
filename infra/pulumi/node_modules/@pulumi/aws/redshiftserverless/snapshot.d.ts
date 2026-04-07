import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a new Amazon Redshift Serverless Snapshot.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshiftserverless.Snapshot("example", {
 *     namespaceName: exampleAwsRedshiftserverlessWorkgroup.namespaceName,
 *     snapshotName: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Serverless Snapshots using the `snapshot_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshiftserverless/snapshot:Snapshot example example
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
     * All of the Amazon Web Services accounts that have access to restore a snapshot to a provisioned cluster.
     */
    readonly accountsWithProvisionedRestoreAccesses: pulumi.Output<string[]>;
    /**
     * All of the Amazon Web Services accounts that have access to restore a snapshot to a namespace.
     */
    readonly accountsWithRestoreAccesses: pulumi.Output<string[]>;
    /**
     * The username of the database within a snapshot.
     */
    readonly adminUsername: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the snapshot.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The unique identifier of the KMS key used to encrypt the snapshot.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the namespace the snapshot was created from.
     */
    readonly namespaceArn: pulumi.Output<string>;
    /**
     * The namespace to create a snapshot for.
     */
    readonly namespaceName: pulumi.Output<string>;
    /**
     * The owner Amazon Web Services; account of the snapshot.
     */
    readonly ownerAccount: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * How long to retain the created snapshot. Default value is `-1`.
     */
    readonly retentionPeriod: pulumi.Output<number | undefined>;
    /**
     * The name of the snapshot.
     */
    readonly snapshotName: pulumi.Output<string>;
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
     * All of the Amazon Web Services accounts that have access to restore a snapshot to a provisioned cluster.
     */
    accountsWithProvisionedRestoreAccesses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * All of the Amazon Web Services accounts that have access to restore a snapshot to a namespace.
     */
    accountsWithRestoreAccesses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The username of the database within a snapshot.
     */
    adminUsername?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the snapshot.
     */
    arn?: pulumi.Input<string>;
    /**
     * The unique identifier of the KMS key used to encrypt the snapshot.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the namespace the snapshot was created from.
     */
    namespaceArn?: pulumi.Input<string>;
    /**
     * The namespace to create a snapshot for.
     */
    namespaceName?: pulumi.Input<string>;
    /**
     * The owner Amazon Web Services; account of the snapshot.
     */
    ownerAccount?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * How long to retain the created snapshot. Default value is `-1`.
     */
    retentionPeriod?: pulumi.Input<number>;
    /**
     * The name of the snapshot.
     */
    snapshotName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Snapshot resource.
 */
export interface SnapshotArgs {
    /**
     * The namespace to create a snapshot for.
     */
    namespaceName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * How long to retain the created snapshot. Default value is `-1`.
     */
    retentionPeriod?: pulumi.Input<number>;
    /**
     * The name of the snapshot.
     */
    snapshotName: pulumi.Input<string>;
}
