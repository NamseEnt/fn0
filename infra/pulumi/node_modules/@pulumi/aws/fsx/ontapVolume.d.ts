import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a FSx ONTAP Volume.
 * See the [FSx ONTAP User Guide](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/managing-volumes.html) for more information.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.fsx.OntapVolume("test", {
 *     name: "test",
 *     junctionPath: "/test",
 *     sizeInMegabytes: 1024,
 *     storageEfficiencyEnabled: true,
 *     storageVirtualMachineId: testAwsFsxOntapStorageVirtualMachine.id,
 * });
 * ```
 *
 * ### Using Tiering Policy
 *
 * Additional information on tiering policy with ONTAP Volumes can be found in the [FSx ONTAP Guide](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/managing-volumes.html).
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.fsx.OntapVolume("test", {
 *     name: "test",
 *     junctionPath: "/test",
 *     sizeInMegabytes: 1024,
 *     storageEfficiencyEnabled: true,
 *     storageVirtualMachineId: testAwsFsxOntapStorageVirtualMachine.id,
 *     tieringPolicy: {
 *         name: "AUTO",
 *         coolingPeriod: 31,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FSx ONTAP volume using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fsx/ontapVolume:OntapVolume example fsvol-12345678abcdef123
 * ```
 */
export declare class OntapVolume extends pulumi.CustomResource {
    /**
     * Get an existing OntapVolume resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OntapVolumeState, opts?: pulumi.CustomResourceOptions): OntapVolume;
    /**
     * Returns true if the given object is an instance of OntapVolume.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OntapVolume;
    /**
     * The Aggregate configuration only applies to `FLEXGROUP` volumes. See [`aggregateConfiguration` Block] for details.
     */
    readonly aggregateConfiguration: pulumi.Output<outputs.fsx.OntapVolumeAggregateConfiguration | undefined>;
    /**
     * Amazon Resource Name of the volune.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Setting this to `true` allows a SnapLock administrator to delete an FSx for ONTAP SnapLock Enterprise volume with unexpired write once, read many (WORM) files. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    readonly bypassSnaplockEnterpriseRetention: pulumi.Output<boolean | undefined>;
    /**
     * A boolean flag indicating whether tags for the volume should be copied to backups. This value defaults to `false`.
     */
    readonly copyTagsToBackups: pulumi.Output<boolean | undefined>;
    /**
     * Describes the file system for the volume, e.g. `fs-12345679`
     */
    readonly fileSystemId: pulumi.Output<string>;
    /**
     * A map of tags to apply to the volume's final backup.
     */
    readonly finalBackupTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Specifies the FlexCache endpoint type of the volume, Valid values are `NONE`, `ORIGIN`, `CACHE`. Default value is `NONE`. These can be set by the ONTAP CLI or API and are use with FlexCache feature.
     */
    readonly flexcacheEndpointType: pulumi.Output<string>;
    /**
     * Specifies the location in the storage virtual machine's namespace where the volume is mounted. The junctionPath must have a leading forward slash, such as `/vol3`
     */
    readonly junctionPath: pulumi.Output<string | undefined>;
    /**
     * The name of the Volume. You can use a maximum of 203 alphanumeric characters, plus the underscore (_) special character.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Specifies the type of volume, valid values are `RW`, `DP`. Default value is `RW`. These can be set by the ONTAP CLI or API. This setting is used as part of migration and replication [Migrating to Amazon FSx for NetApp ONTAP](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/migrating-fsx-ontap.html)
     */
    readonly ontapVolumeType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the volume security style, Valid values are `UNIX`, `NTFS`, and `MIXED`.
     */
    readonly securityStyle: pulumi.Output<string>;
    /**
     * Specifies the size of the volume, in megabytes (MB), that you are creating. Can be used for any size but required for volumes over 2 PB. Either sizeInBytes or sizeInMegabytes must be specified. Minimum size for `FLEXGROUP` volumes are 100GiB per constituent.
     */
    readonly sizeInBytes: pulumi.Output<string>;
    /**
     * Specifies the size of the volume, in megabytes (MB), that you are creating. Supported when creating volumes under 2 PB. Either sizeInBytes or sizeInMegabytes must be specified. Minimum size for `FLEXGROUP` volumes are 100GiB per constituent.
     */
    readonly sizeInMegabytes: pulumi.Output<number>;
    /**
     * When enabled, will skip the default final backup taken when the volume is deleted. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    readonly skipFinalBackup: pulumi.Output<boolean | undefined>;
    /**
     * The SnapLock configuration for an FSx for ONTAP volume. See `snaplockConfiguration` Block for details.
     */
    readonly snaplockConfiguration: pulumi.Output<outputs.fsx.OntapVolumeSnaplockConfiguration | undefined>;
    /**
     * Specifies the snapshot policy for the volume. See [snapshot policies](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/snapshots-ontap.html#snapshot-policies) in the Amazon FSx ONTAP User Guide
     */
    readonly snapshotPolicy: pulumi.Output<string>;
    /**
     * Set to true to enable deduplication, compression, and compaction storage efficiency features on the volume.
     */
    readonly storageEfficiencyEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Specifies the storage virtual machine in which to create the volume.
     *
     * The following arguments are optional:
     */
    readonly storageVirtualMachineId: pulumi.Output<string>;
    /**
     * A map of tags to assign to the volume. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The data tiering policy for an FSx for ONTAP volume. See `tieringPolicy` Block for details.
     */
    readonly tieringPolicy: pulumi.Output<outputs.fsx.OntapVolumeTieringPolicy | undefined>;
    /**
     * The Volume's UUID (universally unique identifier).
     */
    readonly uuid: pulumi.Output<string>;
    /**
     * Specifies the styles of volume, valid values are `FLEXVOL`, `FLEXGROUP`. Default value is `FLEXVOL`. FLEXGROUPS have a larger minimum and maximum size. See Volume Styles for more details. [Volume Styles](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/volume-styles.html)
     */
    readonly volumeStyle: pulumi.Output<string>;
    /**
     * The type of volume, currently the only valid value is `ONTAP`.
     */
    readonly volumeType: pulumi.Output<string | undefined>;
    /**
     * Create a OntapVolume resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OntapVolumeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OntapVolume resources.
 */
export interface OntapVolumeState {
    /**
     * The Aggregate configuration only applies to `FLEXGROUP` volumes. See [`aggregateConfiguration` Block] for details.
     */
    aggregateConfiguration?: pulumi.Input<inputs.fsx.OntapVolumeAggregateConfiguration>;
    /**
     * Amazon Resource Name of the volune.
     */
    arn?: pulumi.Input<string>;
    /**
     * Setting this to `true` allows a SnapLock administrator to delete an FSx for ONTAP SnapLock Enterprise volume with unexpired write once, read many (WORM) files. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    bypassSnaplockEnterpriseRetention?: pulumi.Input<boolean>;
    /**
     * A boolean flag indicating whether tags for the volume should be copied to backups. This value defaults to `false`.
     */
    copyTagsToBackups?: pulumi.Input<boolean>;
    /**
     * Describes the file system for the volume, e.g. `fs-12345679`
     */
    fileSystemId?: pulumi.Input<string>;
    /**
     * A map of tags to apply to the volume's final backup.
     */
    finalBackupTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the FlexCache endpoint type of the volume, Valid values are `NONE`, `ORIGIN`, `CACHE`. Default value is `NONE`. These can be set by the ONTAP CLI or API and are use with FlexCache feature.
     */
    flexcacheEndpointType?: pulumi.Input<string>;
    /**
     * Specifies the location in the storage virtual machine's namespace where the volume is mounted. The junctionPath must have a leading forward slash, such as `/vol3`
     */
    junctionPath?: pulumi.Input<string>;
    /**
     * The name of the Volume. You can use a maximum of 203 alphanumeric characters, plus the underscore (_) special character.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies the type of volume, valid values are `RW`, `DP`. Default value is `RW`. These can be set by the ONTAP CLI or API. This setting is used as part of migration and replication [Migrating to Amazon FSx for NetApp ONTAP](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/migrating-fsx-ontap.html)
     */
    ontapVolumeType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the volume security style, Valid values are `UNIX`, `NTFS`, and `MIXED`.
     */
    securityStyle?: pulumi.Input<string>;
    /**
     * Specifies the size of the volume, in megabytes (MB), that you are creating. Can be used for any size but required for volumes over 2 PB. Either sizeInBytes or sizeInMegabytes must be specified. Minimum size for `FLEXGROUP` volumes are 100GiB per constituent.
     */
    sizeInBytes?: pulumi.Input<string>;
    /**
     * Specifies the size of the volume, in megabytes (MB), that you are creating. Supported when creating volumes under 2 PB. Either sizeInBytes or sizeInMegabytes must be specified. Minimum size for `FLEXGROUP` volumes are 100GiB per constituent.
     */
    sizeInMegabytes?: pulumi.Input<number>;
    /**
     * When enabled, will skip the default final backup taken when the volume is deleted. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    skipFinalBackup?: pulumi.Input<boolean>;
    /**
     * The SnapLock configuration for an FSx for ONTAP volume. See `snaplockConfiguration` Block for details.
     */
    snaplockConfiguration?: pulumi.Input<inputs.fsx.OntapVolumeSnaplockConfiguration>;
    /**
     * Specifies the snapshot policy for the volume. See [snapshot policies](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/snapshots-ontap.html#snapshot-policies) in the Amazon FSx ONTAP User Guide
     */
    snapshotPolicy?: pulumi.Input<string>;
    /**
     * Set to true to enable deduplication, compression, and compaction storage efficiency features on the volume.
     */
    storageEfficiencyEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies the storage virtual machine in which to create the volume.
     *
     * The following arguments are optional:
     */
    storageVirtualMachineId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the volume. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The data tiering policy for an FSx for ONTAP volume. See `tieringPolicy` Block for details.
     */
    tieringPolicy?: pulumi.Input<inputs.fsx.OntapVolumeTieringPolicy>;
    /**
     * The Volume's UUID (universally unique identifier).
     */
    uuid?: pulumi.Input<string>;
    /**
     * Specifies the styles of volume, valid values are `FLEXVOL`, `FLEXGROUP`. Default value is `FLEXVOL`. FLEXGROUPS have a larger minimum and maximum size. See Volume Styles for more details. [Volume Styles](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/volume-styles.html)
     */
    volumeStyle?: pulumi.Input<string>;
    /**
     * The type of volume, currently the only valid value is `ONTAP`.
     */
    volumeType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OntapVolume resource.
 */
export interface OntapVolumeArgs {
    /**
     * The Aggregate configuration only applies to `FLEXGROUP` volumes. See [`aggregateConfiguration` Block] for details.
     */
    aggregateConfiguration?: pulumi.Input<inputs.fsx.OntapVolumeAggregateConfiguration>;
    /**
     * Setting this to `true` allows a SnapLock administrator to delete an FSx for ONTAP SnapLock Enterprise volume with unexpired write once, read many (WORM) files. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    bypassSnaplockEnterpriseRetention?: pulumi.Input<boolean>;
    /**
     * A boolean flag indicating whether tags for the volume should be copied to backups. This value defaults to `false`.
     */
    copyTagsToBackups?: pulumi.Input<boolean>;
    /**
     * A map of tags to apply to the volume's final backup.
     */
    finalBackupTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the location in the storage virtual machine's namespace where the volume is mounted. The junctionPath must have a leading forward slash, such as `/vol3`
     */
    junctionPath?: pulumi.Input<string>;
    /**
     * The name of the Volume. You can use a maximum of 203 alphanumeric characters, plus the underscore (_) special character.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies the type of volume, valid values are `RW`, `DP`. Default value is `RW`. These can be set by the ONTAP CLI or API. This setting is used as part of migration and replication [Migrating to Amazon FSx for NetApp ONTAP](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/migrating-fsx-ontap.html)
     */
    ontapVolumeType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the volume security style, Valid values are `UNIX`, `NTFS`, and `MIXED`.
     */
    securityStyle?: pulumi.Input<string>;
    /**
     * Specifies the size of the volume, in megabytes (MB), that you are creating. Can be used for any size but required for volumes over 2 PB. Either sizeInBytes or sizeInMegabytes must be specified. Minimum size for `FLEXGROUP` volumes are 100GiB per constituent.
     */
    sizeInBytes?: pulumi.Input<string>;
    /**
     * Specifies the size of the volume, in megabytes (MB), that you are creating. Supported when creating volumes under 2 PB. Either sizeInBytes or sizeInMegabytes must be specified. Minimum size for `FLEXGROUP` volumes are 100GiB per constituent.
     */
    sizeInMegabytes?: pulumi.Input<number>;
    /**
     * When enabled, will skip the default final backup taken when the volume is deleted. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    skipFinalBackup?: pulumi.Input<boolean>;
    /**
     * The SnapLock configuration for an FSx for ONTAP volume. See `snaplockConfiguration` Block for details.
     */
    snaplockConfiguration?: pulumi.Input<inputs.fsx.OntapVolumeSnaplockConfiguration>;
    /**
     * Specifies the snapshot policy for the volume. See [snapshot policies](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/snapshots-ontap.html#snapshot-policies) in the Amazon FSx ONTAP User Guide
     */
    snapshotPolicy?: pulumi.Input<string>;
    /**
     * Set to true to enable deduplication, compression, and compaction storage efficiency features on the volume.
     */
    storageEfficiencyEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies the storage virtual machine in which to create the volume.
     *
     * The following arguments are optional:
     */
    storageVirtualMachineId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the volume. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The data tiering policy for an FSx for ONTAP volume. See `tieringPolicy` Block for details.
     */
    tieringPolicy?: pulumi.Input<inputs.fsx.OntapVolumeTieringPolicy>;
    /**
     * Specifies the styles of volume, valid values are `FLEXVOL`, `FLEXGROUP`. Default value is `FLEXVOL`. FLEXGROUPS have a larger minimum and maximum size. See Volume Styles for more details. [Volume Styles](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/volume-styles.html)
     */
    volumeStyle?: pulumi.Input<string>;
    /**
     * The type of volume, currently the only valid value is `ONTAP`.
     */
    volumeType?: pulumi.Input<string>;
}
