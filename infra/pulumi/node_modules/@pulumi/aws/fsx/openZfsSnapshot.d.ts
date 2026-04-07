import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon FSx for OpenZFS volume.
 * See the [FSx OpenZFS User Guide](https://docs.aws.amazon.com/fsx/latest/OpenZFSGuide/what-is-fsx.html) for more information.
 *
 * ## Example Usage
 *
 * ### Root volume Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleOpenZfsFileSystem = new aws.fsx.OpenZfsFileSystem("example", {
 *     storageCapacity: 64,
 *     subnetIds: [exampleAwsSubnet.id],
 *     deploymentType: "SINGLE_AZ_1",
 *     throughputCapacity: 64,
 * });
 * const example = new aws.fsx.OpenZfsSnapshot("example", {
 *     name: "example",
 *     volumeId: exampleOpenZfsFileSystem.rootVolumeId,
 * });
 * ```
 *
 * ### Child volume Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleOpenZfsFileSystem = new aws.fsx.OpenZfsFileSystem("example", {
 *     storageCapacity: 64,
 *     subnetIds: [exampleAwsSubnet.id],
 *     deploymentType: "SINGLE_AZ_1",
 *     throughputCapacity: 64,
 * });
 * const exampleOpenZfsVolume = new aws.fsx.OpenZfsVolume("example", {
 *     name: "example",
 *     parentVolumeId: exampleOpenZfsFileSystem.rootVolumeId,
 * });
 * const example = new aws.fsx.OpenZfsSnapshot("example", {
 *     name: "example",
 *     volumeId: exampleOpenZfsVolume.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FSx OpenZFS snapshot using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fsx/openZfsSnapshot:OpenZfsSnapshot example fs-543ab12b1ca672f33
 * ```
 */
export declare class OpenZfsSnapshot extends pulumi.CustomResource {
    /**
     * Get an existing OpenZfsSnapshot resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OpenZfsSnapshotState, opts?: pulumi.CustomResourceOptions): OpenZfsSnapshot;
    /**
     * Returns true if the given object is an instance of OpenZfsSnapshot.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OpenZfsSnapshot;
    /**
     * Amazon Resource Name of the snapshot.
     */
    readonly arn: pulumi.Output<string>;
    readonly creationTime: pulumi.Output<string>;
    /**
     * The name of the Snapshot. You can use a maximum of 203 alphanumeric characters plus either _ or -  or : or . for the name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the file system. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level. If you have set `copyTagsToBackups` to true, and you specify one or more tags, no existing file system tags are copied from the file system to the backup.
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
     * The ID of the volume to snapshot. This can be the root volume or a child volume.
     */
    readonly volumeId: pulumi.Output<string>;
    /**
     * Create a OpenZfsSnapshot resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OpenZfsSnapshotArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OpenZfsSnapshot resources.
 */
export interface OpenZfsSnapshotState {
    /**
     * Amazon Resource Name of the snapshot.
     */
    arn?: pulumi.Input<string>;
    creationTime?: pulumi.Input<string>;
    /**
     * The name of the Snapshot. You can use a maximum of 203 alphanumeric characters plus either _ or -  or : or . for the name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the file system. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level. If you have set `copyTagsToBackups` to true, and you specify one or more tags, no existing file system tags are copied from the file system to the backup.
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
     * The ID of the volume to snapshot. This can be the root volume or a child volume.
     */
    volumeId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OpenZfsSnapshot resource.
 */
export interface OpenZfsSnapshotArgs {
    /**
     * The name of the Snapshot. You can use a maximum of 203 alphanumeric characters plus either _ or -  or : or . for the name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the file system. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level. If you have set `copyTagsToBackups` to true, and you specify one or more tags, no existing file system tags are copied from the file system to the backup.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the volume to snapshot. This can be the root volume or a child volume.
     */
    volumeId: pulumi.Input<string>;
}
