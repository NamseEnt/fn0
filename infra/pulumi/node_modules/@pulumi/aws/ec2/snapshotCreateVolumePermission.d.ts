import * as pulumi from "@pulumi/pulumi";
/**
 * Adds permission to create volumes off of a given EBS Snapshot.
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
 * });
 * const exampleSnapshot = new aws.ebs.Snapshot("example_snapshot", {volumeId: example.id});
 * const examplePerm = new aws.ec2.SnapshotCreateVolumePermission("example_perm", {
 *     snapshotId: exampleSnapshot.id,
 *     accountId: "12345678",
 * });
 * ```
 */
export declare class SnapshotCreateVolumePermission extends pulumi.CustomResource {
    /**
     * Get an existing SnapshotCreateVolumePermission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SnapshotCreateVolumePermissionState, opts?: pulumi.CustomResourceOptions): SnapshotCreateVolumePermission;
    /**
     * Returns true if the given object is an instance of SnapshotCreateVolumePermission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SnapshotCreateVolumePermission;
    /**
     * An AWS Account ID to add create volume permissions. The AWS Account cannot be the snapshot's owner
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A snapshot ID
     */
    readonly snapshotId: pulumi.Output<string>;
    /**
     * Create a SnapshotCreateVolumePermission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SnapshotCreateVolumePermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SnapshotCreateVolumePermission resources.
 */
export interface SnapshotCreateVolumePermissionState {
    /**
     * An AWS Account ID to add create volume permissions. The AWS Account cannot be the snapshot's owner
     */
    accountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A snapshot ID
     */
    snapshotId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SnapshotCreateVolumePermission resource.
 */
export interface SnapshotCreateVolumePermissionArgs {
    /**
     * An AWS Account ID to add create volume permissions. The AWS Account cannot be the snapshot's owner
     */
    accountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A snapshot ID
     */
    snapshotId: pulumi.Input<string>;
}
