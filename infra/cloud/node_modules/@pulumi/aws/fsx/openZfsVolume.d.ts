import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon FSx for OpenZFS volume.
 * See the [FSx OpenZFS User Guide](https://docs.aws.amazon.com/fsx/latest/OpenZFSGuide/what-is-fsx.html) for more information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.fsx.OpenZfsVolume("test", {
 *     name: "testvolume",
 *     parentVolumeId: testAwsFsxOpenzfsFileSystem.rootVolumeId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FSx Volumes using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fsx/openZfsVolume:OpenZfsVolume example fsvol-543ab12b1ca672f33
 * ```
 */
export declare class OpenZfsVolume extends pulumi.CustomResource {
    /**
     * Get an existing OpenZfsVolume resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OpenZfsVolumeState, opts?: pulumi.CustomResourceOptions): OpenZfsVolume;
    /**
     * Returns true if the given object is an instance of OpenZfsVolume.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OpenZfsVolume;
    /**
     * Amazon Resource Name of the file system.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to snapshots. The default value is false.
     */
    readonly copyTagsToSnapshots: pulumi.Output<boolean | undefined>;
    /**
     * Method used to compress the data on the volume. Valid values are `NONE` or `ZSTD`. Child volumes that don't specify compression option will inherit from parent volume. This option on file system applies to the root volume.
     */
    readonly dataCompressionType: pulumi.Output<string | undefined>;
    /**
     * Whether to delete all child volumes and snapshots. Valid values: `DELETE_CHILD_VOLUMES_AND_SNAPSHOTS`. This configuration must be applied separately before attempting to delete the resource to have the desired behavior..
     */
    readonly deleteVolumeOptions: pulumi.Output<string | undefined>;
    /**
     * The name of the Volume. You can use a maximum of 203 alphanumeric characters, plus the underscore (_) special character.
     */
    readonly name: pulumi.Output<string>;
    /**
     * NFS export configuration for the root volume. Exactly 1 item. See `nfsExports` Block Below for details.
     */
    readonly nfsExports: pulumi.Output<outputs.fsx.OpenZfsVolumeNfsExports | undefined>;
    /**
     * Specifies the configuration to use when creating the OpenZFS volume. See `originSnapshot` Block below for details.
     */
    readonly originSnapshot: pulumi.Output<outputs.fsx.OpenZfsVolumeOriginSnapshot | undefined>;
    /**
     * The volume id of volume that will be the parent volume for the volume being created, this could be the root volume created from the `aws.fsx.OpenZfsFileSystem` resource with the `rootVolumeId` or the `id` property of another `aws.fsx.OpenZfsVolume`.
     */
    readonly parentVolumeId: pulumi.Output<string>;
    /**
     * specifies whether the volume is read-only. Default is false.
     */
    readonly readOnly: pulumi.Output<boolean>;
    /**
     * The record size of an OpenZFS volume, in kibibytes (KiB). Valid values are `4`, `8`, `16`, `32`, `64`, `128`, `256`, `512`, or `1024` KiB. The default is `128` KiB.
     */
    readonly recordSizeKib: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The maximum amount of storage in gibibytes (GiB) that the volume can use from its parent.
     */
    readonly storageCapacityQuotaGib: pulumi.Output<number>;
    /**
     * The amount of storage in gibibytes (GiB) to reserve from the parent volume.
     */
    readonly storageCapacityReservationGib: pulumi.Output<number>;
    /**
     * A map of tags to assign to the file system. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Specify how much storage users or groups can use on the volume. Maximum number of items defined by [FSx for OpenZFS Resource quota](https://docs.aws.amazon.com/fsx/latest/OpenZFSGuide/limits.html#limits-openzfs-resources-file-system). See `userAndGroupQuotas` Block Below.
     */
    readonly userAndGroupQuotas: pulumi.Output<outputs.fsx.OpenZfsVolumeUserAndGroupQuota[]>;
    readonly volumeType: pulumi.Output<string | undefined>;
    /**
     * Create a OpenZfsVolume resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OpenZfsVolumeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OpenZfsVolume resources.
 */
export interface OpenZfsVolumeState {
    /**
     * Amazon Resource Name of the file system.
     */
    arn?: pulumi.Input<string>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to snapshots. The default value is false.
     */
    copyTagsToSnapshots?: pulumi.Input<boolean>;
    /**
     * Method used to compress the data on the volume. Valid values are `NONE` or `ZSTD`. Child volumes that don't specify compression option will inherit from parent volume. This option on file system applies to the root volume.
     */
    dataCompressionType?: pulumi.Input<string>;
    /**
     * Whether to delete all child volumes and snapshots. Valid values: `DELETE_CHILD_VOLUMES_AND_SNAPSHOTS`. This configuration must be applied separately before attempting to delete the resource to have the desired behavior..
     */
    deleteVolumeOptions?: pulumi.Input<string>;
    /**
     * The name of the Volume. You can use a maximum of 203 alphanumeric characters, plus the underscore (_) special character.
     */
    name?: pulumi.Input<string>;
    /**
     * NFS export configuration for the root volume. Exactly 1 item. See `nfsExports` Block Below for details.
     */
    nfsExports?: pulumi.Input<inputs.fsx.OpenZfsVolumeNfsExports>;
    /**
     * Specifies the configuration to use when creating the OpenZFS volume. See `originSnapshot` Block below for details.
     */
    originSnapshot?: pulumi.Input<inputs.fsx.OpenZfsVolumeOriginSnapshot>;
    /**
     * The volume id of volume that will be the parent volume for the volume being created, this could be the root volume created from the `aws.fsx.OpenZfsFileSystem` resource with the `rootVolumeId` or the `id` property of another `aws.fsx.OpenZfsVolume`.
     */
    parentVolumeId?: pulumi.Input<string>;
    /**
     * specifies whether the volume is read-only. Default is false.
     */
    readOnly?: pulumi.Input<boolean>;
    /**
     * The record size of an OpenZFS volume, in kibibytes (KiB). Valid values are `4`, `8`, `16`, `32`, `64`, `128`, `256`, `512`, or `1024` KiB. The default is `128` KiB.
     */
    recordSizeKib?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The maximum amount of storage in gibibytes (GiB) that the volume can use from its parent.
     */
    storageCapacityQuotaGib?: pulumi.Input<number>;
    /**
     * The amount of storage in gibibytes (GiB) to reserve from the parent volume.
     */
    storageCapacityReservationGib?: pulumi.Input<number>;
    /**
     * A map of tags to assign to the file system. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Specify how much storage users or groups can use on the volume. Maximum number of items defined by [FSx for OpenZFS Resource quota](https://docs.aws.amazon.com/fsx/latest/OpenZFSGuide/limits.html#limits-openzfs-resources-file-system). See `userAndGroupQuotas` Block Below.
     */
    userAndGroupQuotas?: pulumi.Input<pulumi.Input<inputs.fsx.OpenZfsVolumeUserAndGroupQuota>[]>;
    volumeType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OpenZfsVolume resource.
 */
export interface OpenZfsVolumeArgs {
    /**
     * A boolean flag indicating whether tags for the file system should be copied to snapshots. The default value is false.
     */
    copyTagsToSnapshots?: pulumi.Input<boolean>;
    /**
     * Method used to compress the data on the volume. Valid values are `NONE` or `ZSTD`. Child volumes that don't specify compression option will inherit from parent volume. This option on file system applies to the root volume.
     */
    dataCompressionType?: pulumi.Input<string>;
    /**
     * Whether to delete all child volumes and snapshots. Valid values: `DELETE_CHILD_VOLUMES_AND_SNAPSHOTS`. This configuration must be applied separately before attempting to delete the resource to have the desired behavior..
     */
    deleteVolumeOptions?: pulumi.Input<string>;
    /**
     * The name of the Volume. You can use a maximum of 203 alphanumeric characters, plus the underscore (_) special character.
     */
    name?: pulumi.Input<string>;
    /**
     * NFS export configuration for the root volume. Exactly 1 item. See `nfsExports` Block Below for details.
     */
    nfsExports?: pulumi.Input<inputs.fsx.OpenZfsVolumeNfsExports>;
    /**
     * Specifies the configuration to use when creating the OpenZFS volume. See `originSnapshot` Block below for details.
     */
    originSnapshot?: pulumi.Input<inputs.fsx.OpenZfsVolumeOriginSnapshot>;
    /**
     * The volume id of volume that will be the parent volume for the volume being created, this could be the root volume created from the `aws.fsx.OpenZfsFileSystem` resource with the `rootVolumeId` or the `id` property of another `aws.fsx.OpenZfsVolume`.
     */
    parentVolumeId: pulumi.Input<string>;
    /**
     * specifies whether the volume is read-only. Default is false.
     */
    readOnly?: pulumi.Input<boolean>;
    /**
     * The record size of an OpenZFS volume, in kibibytes (KiB). Valid values are `4`, `8`, `16`, `32`, `64`, `128`, `256`, `512`, or `1024` KiB. The default is `128` KiB.
     */
    recordSizeKib?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The maximum amount of storage in gibibytes (GiB) that the volume can use from its parent.
     */
    storageCapacityQuotaGib?: pulumi.Input<number>;
    /**
     * The amount of storage in gibibytes (GiB) to reserve from the parent volume.
     */
    storageCapacityReservationGib?: pulumi.Input<number>;
    /**
     * A map of tags to assign to the file system. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specify how much storage users or groups can use on the volume. Maximum number of items defined by [FSx for OpenZFS Resource quota](https://docs.aws.amazon.com/fsx/latest/OpenZFSGuide/limits.html#limits-openzfs-resources-file-system). See `userAndGroupQuotas` Block Below.
     */
    userAndGroupQuotas?: pulumi.Input<pulumi.Input<inputs.fsx.OpenZfsVolumeUserAndGroupQuota>[]>;
    volumeType?: pulumi.Input<string>;
}
