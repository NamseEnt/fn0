import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.redshift.Cluster("default", {
 *     clusterIdentifier: "tf-redshift-cluster",
 *     databaseName: "mydb",
 *     masterUsername: "foo",
 *     masterPassword: "Mustbe8characters",
 *     nodeType: "dc1.large",
 *     clusterType: "single-node",
 * });
 * const defaultSnapshotSchedule = new aws.redshift.SnapshotSchedule("default", {
 *     identifier: "tf-redshift-snapshot-schedule",
 *     definitions: ["rate(12 hours)"],
 * });
 * const defaultSnapshotScheduleAssociation = new aws.redshift.SnapshotScheduleAssociation("default", {
 *     clusterIdentifier: _default.id,
 *     scheduleIdentifier: defaultSnapshotSchedule.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Snapshot Schedule Association using the `<cluster-identifier>/<schedule-identifier>`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/snapshotScheduleAssociation:SnapshotScheduleAssociation default tf-redshift-cluster/tf-redshift-snapshot-schedule
 * ```
 */
export declare class SnapshotScheduleAssociation extends pulumi.CustomResource {
    /**
     * Get an existing SnapshotScheduleAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SnapshotScheduleAssociationState, opts?: pulumi.CustomResourceOptions): SnapshotScheduleAssociation;
    /**
     * Returns true if the given object is an instance of SnapshotScheduleAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SnapshotScheduleAssociation;
    /**
     * The cluster identifier.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The snapshot schedule identifier.
     */
    readonly scheduleIdentifier: pulumi.Output<string>;
    /**
     * Create a SnapshotScheduleAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SnapshotScheduleAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SnapshotScheduleAssociation resources.
 */
export interface SnapshotScheduleAssociationState {
    /**
     * The cluster identifier.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The snapshot schedule identifier.
     */
    scheduleIdentifier?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SnapshotScheduleAssociation resource.
 */
export interface SnapshotScheduleAssociationArgs {
    /**
     * The cluster identifier.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The snapshot schedule identifier.
     */
    scheduleIdentifier: pulumi.Input<string>;
}
