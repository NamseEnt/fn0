import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon FSx for OpenZFS file system.
 * See the [FSx OpenZFS User Guide](https://docs.aws.amazon.com/fsx/latest/OpenZFSGuide/what-is-fsx.html) for more information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.fsx.OpenZfsFileSystem("test", {
 *     storageCapacity: 64,
 *     subnetIds: [test1.id],
 *     deploymentType: "SINGLE_AZ_1",
 *     throughputCapacity: 64,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FSx File Systems using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fsx/openZfsFileSystem:OpenZfsFileSystem example fs-543ab12b1ca672f33
 * ```
 * Certain resource arguments, like `security_group_ids`, do not have a FSx API method for reading the information after creation. If the argument is set in the Pulumi program on an imported resource, Pulumi will always show a difference. To workaround this behavior, either omit the argument from the Pulumi program or use `ignore_changes` to hide the difference. For example:
 */
export declare class OpenZfsFileSystem extends pulumi.CustomResource {
    /**
     * Get an existing OpenZfsFileSystem resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OpenZfsFileSystemState, opts?: pulumi.CustomResourceOptions): OpenZfsFileSystem;
    /**
     * Returns true if the given object is an instance of OpenZfsFileSystem.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OpenZfsFileSystem;
    /**
     * Amazon Resource Name of the file system.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The number of days to retain automatic backups. Setting this to 0 disables automatic backups. You can retain automatic backups for a maximum of 90 days.
     */
    readonly automaticBackupRetentionDays: pulumi.Output<number | undefined>;
    /**
     * The ID of the source backup to create the filesystem from.
     */
    readonly backupId: pulumi.Output<string | undefined>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to backups. The default value is false.
     */
    readonly copyTagsToBackups: pulumi.Output<boolean | undefined>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to snapshots. The default value is false.
     */
    readonly copyTagsToVolumes: pulumi.Output<boolean | undefined>;
    /**
     * A recurring daily time, in the format HH:MM. HH is the zero-padded hour of the day (0-23), and MM is the zero-padded minute of the hour. For example, 05:00 specifies 5 AM daily. Requires `automaticBackupRetentionDays` to be set.
     */
    readonly dailyAutomaticBackupStartTime: pulumi.Output<string>;
    /**
     * List of delete options, which at present supports only one value that specifies whether to delete all child volumes and snapshots when the file system is deleted. Valid values: `DELETE_CHILD_VOLUMES_AND_SNAPSHOTS`.
     */
    readonly deleteOptions: pulumi.Output<string[] | undefined>;
    /**
     * Filesystem deployment type. See the [AWS API documentation](https://docs.aws.amazon.com/fsx/latest/APIReference/API_CreateFileSystemOpenZFSConfiguration.html#FSx-Type-CreateFileSystemOpenZFSConfiguration-DeploymentType) for a list of valid values.
     */
    readonly deploymentType: pulumi.Output<string>;
    /**
     * The SSD IOPS configuration for the Amazon FSx for OpenZFS file system. See `diskIopsConfiguration` Block for details.
     */
    readonly diskIopsConfiguration: pulumi.Output<outputs.fsx.OpenZfsFileSystemDiskIopsConfiguration>;
    /**
     * DNS name for the file system, e.g., `fs-12345678.fsx.us-west-2.amazonaws.com`
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * IP address of the endpoint that is used to access data or to manage the file system.
     */
    readonly endpointIpAddress: pulumi.Output<string>;
    /**
     * (Multi-AZ only) Specifies the IP address range in which the endpoints to access your file system will be created.
     */
    readonly endpointIpAddressRange: pulumi.Output<string>;
    /**
     * A map of tags to apply to the file system's final backup.
     */
    readonly finalBackupTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * ARN for the KMS Key to encrypt the file system at rest, Defaults to an AWS managed KMS Key.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Set of Elastic Network Interface identifiers from which the file system is accessible The first network interface returned is the primary network interface.
     */
    readonly networkInterfaceIds: pulumi.Output<string[]>;
    /**
     * AWS account identifier that created the file system.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * (Multi-AZ only) Required when `deploymentType` is set to `MULTI_AZ_1`. This specifies the subnet in which you want the preferred file server to be located.
     */
    readonly preferredSubnetId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The configuration for the root volume of the file system. All other volumes are children or the root volume. See `rootVolumeConfiguration` Block for details.
     */
    readonly rootVolumeConfiguration: pulumi.Output<outputs.fsx.OpenZfsFileSystemRootVolumeConfiguration>;
    /**
     * Identifier of the root volume, e.g., `fsvol-12345678`
     */
    readonly rootVolumeId: pulumi.Output<string>;
    /**
     * (Multi-AZ only) Specifies the route tables in which Amazon FSx creates the rules for routing traffic to the correct file server. You should specify all virtual private cloud (VPC) route tables associated with the subnets in which your clients are located. By default, Amazon FSx selects your VPC's default route table.
     */
    readonly routeTableIds: pulumi.Output<string[]>;
    /**
     * A list of IDs for the security groups that apply to the specified network interfaces created for file system access. These security groups will apply to all network interfaces.
     */
    readonly securityGroupIds: pulumi.Output<string[] | undefined>;
    /**
     * When enabled, will skip the default final backup taken when the file system is deleted. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    readonly skipFinalBackup: pulumi.Output<boolean | undefined>;
    /**
     * The storage capacity (GiB) of the file system. Valid values between `64` and `524288`.
     */
    readonly storageCapacity: pulumi.Output<number | undefined>;
    /**
     * The filesystem storage type. Only `SSD` is supported.
     */
    readonly storageType: pulumi.Output<string | undefined>;
    /**
     * A list of IDs for the subnets that the file system will be accessible from.
     */
    readonly subnetIds: pulumi.Output<string[]>;
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
     * Throughput (MB/s) of the file system. Valid values depend on `deploymentType`. Must be one of `64`, `128`, `256`, `512`, `1024`, `2048`, `3072`, `4096` for `SINGLE_AZ_1`. Must be one of `160`, `320`, `640`, `1280`, `2560`, `3840`, `5120`, `7680`, `10240` for `SINGLE_AZ_2`.
     *
     * The following arguments are optional:
     */
    readonly throughputCapacity: pulumi.Output<number>;
    /**
     * Identifier of the Virtual Private Cloud for the file system.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * The preferred start time (in `d:HH:MM` format) to perform weekly maintenance, in the UTC time zone.
     */
    readonly weeklyMaintenanceStartTime: pulumi.Output<string>;
    /**
     * Create a OpenZfsFileSystem resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OpenZfsFileSystemArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OpenZfsFileSystem resources.
 */
export interface OpenZfsFileSystemState {
    /**
     * Amazon Resource Name of the file system.
     */
    arn?: pulumi.Input<string>;
    /**
     * The number of days to retain automatic backups. Setting this to 0 disables automatic backups. You can retain automatic backups for a maximum of 90 days.
     */
    automaticBackupRetentionDays?: pulumi.Input<number>;
    /**
     * The ID of the source backup to create the filesystem from.
     */
    backupId?: pulumi.Input<string>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to backups. The default value is false.
     */
    copyTagsToBackups?: pulumi.Input<boolean>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to snapshots. The default value is false.
     */
    copyTagsToVolumes?: pulumi.Input<boolean>;
    /**
     * A recurring daily time, in the format HH:MM. HH is the zero-padded hour of the day (0-23), and MM is the zero-padded minute of the hour. For example, 05:00 specifies 5 AM daily. Requires `automaticBackupRetentionDays` to be set.
     */
    dailyAutomaticBackupStartTime?: pulumi.Input<string>;
    /**
     * List of delete options, which at present supports only one value that specifies whether to delete all child volumes and snapshots when the file system is deleted. Valid values: `DELETE_CHILD_VOLUMES_AND_SNAPSHOTS`.
     */
    deleteOptions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Filesystem deployment type. See the [AWS API documentation](https://docs.aws.amazon.com/fsx/latest/APIReference/API_CreateFileSystemOpenZFSConfiguration.html#FSx-Type-CreateFileSystemOpenZFSConfiguration-DeploymentType) for a list of valid values.
     */
    deploymentType?: pulumi.Input<string>;
    /**
     * The SSD IOPS configuration for the Amazon FSx for OpenZFS file system. See `diskIopsConfiguration` Block for details.
     */
    diskIopsConfiguration?: pulumi.Input<inputs.fsx.OpenZfsFileSystemDiskIopsConfiguration>;
    /**
     * DNS name for the file system, e.g., `fs-12345678.fsx.us-west-2.amazonaws.com`
     */
    dnsName?: pulumi.Input<string>;
    /**
     * IP address of the endpoint that is used to access data or to manage the file system.
     */
    endpointIpAddress?: pulumi.Input<string>;
    /**
     * (Multi-AZ only) Specifies the IP address range in which the endpoints to access your file system will be created.
     */
    endpointIpAddressRange?: pulumi.Input<string>;
    /**
     * A map of tags to apply to the file system's final backup.
     */
    finalBackupTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN for the KMS Key to encrypt the file system at rest, Defaults to an AWS managed KMS Key.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Set of Elastic Network Interface identifiers from which the file system is accessible The first network interface returned is the primary network interface.
     */
    networkInterfaceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * AWS account identifier that created the file system.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * (Multi-AZ only) Required when `deploymentType` is set to `MULTI_AZ_1`. This specifies the subnet in which you want the preferred file server to be located.
     */
    preferredSubnetId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration for the root volume of the file system. All other volumes are children or the root volume. See `rootVolumeConfiguration` Block for details.
     */
    rootVolumeConfiguration?: pulumi.Input<inputs.fsx.OpenZfsFileSystemRootVolumeConfiguration>;
    /**
     * Identifier of the root volume, e.g., `fsvol-12345678`
     */
    rootVolumeId?: pulumi.Input<string>;
    /**
     * (Multi-AZ only) Specifies the route tables in which Amazon FSx creates the rules for routing traffic to the correct file server. You should specify all virtual private cloud (VPC) route tables associated with the subnets in which your clients are located. By default, Amazon FSx selects your VPC's default route table.
     */
    routeTableIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of IDs for the security groups that apply to the specified network interfaces created for file system access. These security groups will apply to all network interfaces.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * When enabled, will skip the default final backup taken when the file system is deleted. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    skipFinalBackup?: pulumi.Input<boolean>;
    /**
     * The storage capacity (GiB) of the file system. Valid values between `64` and `524288`.
     */
    storageCapacity?: pulumi.Input<number>;
    /**
     * The filesystem storage type. Only `SSD` is supported.
     */
    storageType?: pulumi.Input<string>;
    /**
     * A list of IDs for the subnets that the file system will be accessible from.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
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
     * Throughput (MB/s) of the file system. Valid values depend on `deploymentType`. Must be one of `64`, `128`, `256`, `512`, `1024`, `2048`, `3072`, `4096` for `SINGLE_AZ_1`. Must be one of `160`, `320`, `640`, `1280`, `2560`, `3840`, `5120`, `7680`, `10240` for `SINGLE_AZ_2`.
     *
     * The following arguments are optional:
     */
    throughputCapacity?: pulumi.Input<number>;
    /**
     * Identifier of the Virtual Private Cloud for the file system.
     */
    vpcId?: pulumi.Input<string>;
    /**
     * The preferred start time (in `d:HH:MM` format) to perform weekly maintenance, in the UTC time zone.
     */
    weeklyMaintenanceStartTime?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OpenZfsFileSystem resource.
 */
export interface OpenZfsFileSystemArgs {
    /**
     * The number of days to retain automatic backups. Setting this to 0 disables automatic backups. You can retain automatic backups for a maximum of 90 days.
     */
    automaticBackupRetentionDays?: pulumi.Input<number>;
    /**
     * The ID of the source backup to create the filesystem from.
     */
    backupId?: pulumi.Input<string>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to backups. The default value is false.
     */
    copyTagsToBackups?: pulumi.Input<boolean>;
    /**
     * A boolean flag indicating whether tags for the file system should be copied to snapshots. The default value is false.
     */
    copyTagsToVolumes?: pulumi.Input<boolean>;
    /**
     * A recurring daily time, in the format HH:MM. HH is the zero-padded hour of the day (0-23), and MM is the zero-padded minute of the hour. For example, 05:00 specifies 5 AM daily. Requires `automaticBackupRetentionDays` to be set.
     */
    dailyAutomaticBackupStartTime?: pulumi.Input<string>;
    /**
     * List of delete options, which at present supports only one value that specifies whether to delete all child volumes and snapshots when the file system is deleted. Valid values: `DELETE_CHILD_VOLUMES_AND_SNAPSHOTS`.
     */
    deleteOptions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Filesystem deployment type. See the [AWS API documentation](https://docs.aws.amazon.com/fsx/latest/APIReference/API_CreateFileSystemOpenZFSConfiguration.html#FSx-Type-CreateFileSystemOpenZFSConfiguration-DeploymentType) for a list of valid values.
     */
    deploymentType: pulumi.Input<string>;
    /**
     * The SSD IOPS configuration for the Amazon FSx for OpenZFS file system. See `diskIopsConfiguration` Block for details.
     */
    diskIopsConfiguration?: pulumi.Input<inputs.fsx.OpenZfsFileSystemDiskIopsConfiguration>;
    /**
     * (Multi-AZ only) Specifies the IP address range in which the endpoints to access your file system will be created.
     */
    endpointIpAddressRange?: pulumi.Input<string>;
    /**
     * A map of tags to apply to the file system's final backup.
     */
    finalBackupTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN for the KMS Key to encrypt the file system at rest, Defaults to an AWS managed KMS Key.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * (Multi-AZ only) Required when `deploymentType` is set to `MULTI_AZ_1`. This specifies the subnet in which you want the preferred file server to be located.
     */
    preferredSubnetId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration for the root volume of the file system. All other volumes are children or the root volume. See `rootVolumeConfiguration` Block for details.
     */
    rootVolumeConfiguration?: pulumi.Input<inputs.fsx.OpenZfsFileSystemRootVolumeConfiguration>;
    /**
     * (Multi-AZ only) Specifies the route tables in which Amazon FSx creates the rules for routing traffic to the correct file server. You should specify all virtual private cloud (VPC) route tables associated with the subnets in which your clients are located. By default, Amazon FSx selects your VPC's default route table.
     */
    routeTableIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of IDs for the security groups that apply to the specified network interfaces created for file system access. These security groups will apply to all network interfaces.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * When enabled, will skip the default final backup taken when the file system is deleted. This configuration must be applied separately before attempting to delete the resource to have the desired behavior. Defaults to `false`.
     */
    skipFinalBackup?: pulumi.Input<boolean>;
    /**
     * The storage capacity (GiB) of the file system. Valid values between `64` and `524288`.
     */
    storageCapacity?: pulumi.Input<number>;
    /**
     * The filesystem storage type. Only `SSD` is supported.
     */
    storageType?: pulumi.Input<string>;
    /**
     * A list of IDs for the subnets that the file system will be accessible from.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the file system. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Throughput (MB/s) of the file system. Valid values depend on `deploymentType`. Must be one of `64`, `128`, `256`, `512`, `1024`, `2048`, `3072`, `4096` for `SINGLE_AZ_1`. Must be one of `160`, `320`, `640`, `1280`, `2560`, `3840`, `5120`, `7680`, `10240` for `SINGLE_AZ_2`.
     *
     * The following arguments are optional:
     */
    throughputCapacity: pulumi.Input<number>;
    /**
     * The preferred start time (in `d:HH:MM` format) to perform weekly maintenance, in the UTC time zone.
     */
    weeklyMaintenanceStartTime?: pulumi.Input<string>;
}
