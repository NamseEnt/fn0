import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an EBS (Elastic Block Storage) Fast Snapshot Restore.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ebs.FastSnapshotRestore("example", {
 *     availabilityZone: "us-west-2a",
 *     snapshotId: exampleAwsEbsSnapshot.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EC2 (Elastic Compute Cloud) EBS Fast Snapshot Restore using the `availability_zone` and `snapshot_id` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:ebs/fastSnapshotRestore:FastSnapshotRestore example us-west-2a,snap-abcdef123456
 * ```
 */
export declare class FastSnapshotRestore extends pulumi.CustomResource {
    /**
     * Get an existing FastSnapshotRestore resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FastSnapshotRestoreState, opts?: pulumi.CustomResourceOptions): FastSnapshotRestore;
    /**
     * Returns true if the given object is an instance of FastSnapshotRestore.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FastSnapshotRestore;
    /**
     * Availability zone in which to enable fast snapshot restores.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the snapshot.
     */
    readonly snapshotId: pulumi.Output<string>;
    /**
     * State of fast snapshot restores. Valid values are `enabling`, `optimizing`, `enabled`, `disabling`, `disabled`.
     */
    readonly state: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ebs.FastSnapshotRestoreTimeouts | undefined>;
    /**
     * Create a FastSnapshotRestore resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FastSnapshotRestoreArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FastSnapshotRestore resources.
 */
export interface FastSnapshotRestoreState {
    /**
     * Availability zone in which to enable fast snapshot restores.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the snapshot.
     */
    snapshotId?: pulumi.Input<string>;
    /**
     * State of fast snapshot restores. Valid values are `enabling`, `optimizing`, `enabled`, `disabling`, `disabled`.
     */
    state?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ebs.FastSnapshotRestoreTimeouts>;
}
/**
 * The set of arguments for constructing a FastSnapshotRestore resource.
 */
export interface FastSnapshotRestoreArgs {
    /**
     * Availability zone in which to enable fast snapshot restores.
     */
    availabilityZone: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the snapshot.
     */
    snapshotId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ebs.FastSnapshotRestoreTimeouts>;
}
