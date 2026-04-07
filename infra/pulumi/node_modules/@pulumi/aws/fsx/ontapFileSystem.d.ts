import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon FSx for NetApp ONTAP file system.
 * See the [FSx ONTAP User Guide](https://docs.aws.amazon.com/fsx/latest/ONTAPGuide/what-is-fsx-ontap.html) for more information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.fsx.OntapFileSystem("test", {
 *     storageCapacity: 1024,
 *     subnetIds: [
 *         test1.id,
 *         test2.id,
 *     ],
 *     deploymentType: "MULTI_AZ_1",
 *     throughputCapacity: 512,
 *     preferredSubnetId: test1.id,
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testhapairs = new aws.fsx.OntapFileSystem("testhapairs", {
 *     storageCapacity: 2048,
 *     subnetIds: [test1.id],
 *     deploymentType: "SINGLE_AZ_1",
 *     haPairs: 2,
 *     throughputCapacityPerHaPair: 128,
 *     preferredSubnetId: test1.id,
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testsingleazgen2 = new aws.fsx.OntapFileSystem("testsingleazgen2", {
 *     storageCapacity: 4096,
 *     subnetIds: [test1.id],
 *     deploymentType: "SINGLE_AZ_2",
 *     haPairs: 4,
 *     throughputCapacityPerHaPair: 384,
 *     preferredSubnetId: test1.id,
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testmultiazgen2 = new aws.fsx.OntapFileSystem("testmultiazgen2", {
 *     storageCapacity: 1024,
 *     subnetIds: [
 *         test1.id,
 *         test2.id,
 *     ],
 *     deploymentType: "MULTI_AZ_2",
 *     haPairs: 1,
 *     throughputCapacityPerHaPair: 384,
 *     preferredSubnetId: test1.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FSx File Systems using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fsx/ontapFileSystem:OntapFileSystem example fs-543ab12b1ca672f33
 * ```
 * Certain resource arguments, like `security_group_ids`, do not have a FSx API method for reading the information after creation. If the argument is set in the Pulumi program on an imported resource, Pulumi will always show a difference. To workaround this behavior, either omit the argument from the Pulumi program or use `ignore_changes` to hide the difference. For example:
 */
export declare class OntapFileSystem extends pulumi.CustomResource {
    /**
     * Get an existing OntapFileSystem resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OntapFileSystemState, opts?: pulumi.CustomResourceOptions): OntapFileSystem;
    /**
     * Returns true if the given object is an instance of OntapFileSystem.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OntapFileSystem;
    /**
     * Amazon Resource Name of the file system.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The number of days to retain automatic backups. Setting this to 0 disables automatic backups. You can retain automatic backups for a maximum of 90 days.
     */
    readonly automaticBackupRetentionDays: pulumi.Output<number | undefined>;
    /**
     * A recurring daily time, in the format HH:MM. HH is the zero-padded hour of the day (0-23), and MM is the zero-padded minute of the hour. For example, 05:00 specifies 5 AM daily. Requires `automaticBackupRetentionDays` to be set.
     */
    readonly dailyAutomaticBackupStartTime: pulumi.Output<string>;
    /**
     * The filesystem deployment type. Supports `MULTI_AZ_1`, `MULTI_AZ_2`, `SINGLE_AZ_1`, and `SINGLE_AZ_2`.
     */
    readonly deploymentType: pulumi.Output<string>;
    /**
     * The SSD IOPS configuration for the Amazon FSx for NetApp ONTAP file system. See Disk Iops Configuration below.
     */
    readonly diskIopsConfiguration: pulumi.Output<outputs.fsx.OntapFileSystemDiskIopsConfiguration>;
    /**
     * The Domain Name Service (DNS) name for the file system. You can mount your file system using its DNS name.
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * Specifies the IP address range in which the endpoints to access your file system will be created. By default, Amazon FSx selects an unused IP address range for you from the 198.19.* range.
     */
    readonly endpointIpAddressRange: pulumi.Output<string>;
    /**
     * The endpoints that are used to access data or to manage the file system using the NetApp ONTAP CLI, REST API, or NetApp SnapMirror. See Endpoints below.
     */
    readonly endpoints: pulumi.Output<outputs.fsx.OntapFileSystemEndpoint[]>;
    /**
     * The ONTAP administrative password for the fsxadmin user that you can use to administer your file system using the ONTAP CLI and REST API.
     */
    readonly fsxAdminPassword: pulumi.Output<string | undefined>;
    /**
     * The number of haPairs to deploy for the file system. Valid value is 1 for `SINGLE_AZ_1` or `MULTI_AZ_1` and `MULTI_AZ_2`. Valid values are 1 through 12 for `SINGLE_AZ_2`.
     */
    readonly haPairs: pulumi.Output<number>;
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
     * The ID for a subnet. A subnet is a range of IP addresses in your virtual private cloud (VPC).
     */
    readonly preferredSubnetId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the VPC route tables in which your file system's endpoints will be created. You should specify all VPC route tables associated with the subnets in which your clients are located. By default, Amazon FSx selects your VPC's default route table.
     */
    readonly routeTableIds: pulumi.Output<string[]>;
    /**
     * A list of IDs for the security groups that apply to the specified network interfaces created for file system access. These security groups will apply to all network interfaces.
     */
    readonly securityGroupIds: pulumi.Output<string[] | undefined>;
    /**
     * The storage capacity (GiB) of the file system. Valid values between `1024` and `196608` for file systems with deploymentType `SINGLE_AZ_1` and `MULTI_AZ_1`. Valid values are between `1024` and `524288` for `MULTI_AZ_2`. Valid values between `1024` (`1024` per ha pair) and `1048576` for file systems with deploymentType `SINGLE_AZ_2`. For `SINGLE_AZ_2`, the `1048576` (1PB) maximum is only supported when using 2 or more ha_pairs, the maximum is `524288` (512TB) when using 1 ha_pair.
     */
    readonly storageCapacity: pulumi.Output<number>;
    /**
     * The filesystem storage type. defaults to `SSD`.
     */
    readonly storageType: pulumi.Output<string | undefined>;
    /**
     * A list of IDs for the subnets that the file system will be accessible from. Up to 2 subnets can be provided.
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
     * Sets the throughput capacity (in MBps) for the file system that you're creating. Valid values are `128`, `256`, `512`, `1024`, `2048`, and `4096`. This parameter is only supported when not using the haPairs parameter. Either throughputCapacity or throughputCapacityPerHaPair must be specified.
     */
    readonly throughputCapacity: pulumi.Output<number>;
    /**
     * Sets the per-HA-pair throughput capacity (in MBps) for the file system that you're creating, as opposed to `throughputCapacity` which specifies the total throughput capacity for the file system. Valid value for `MULTI_AZ_1` and `SINGLE_AZ_1` are `128`, `256`, `512`, `1024`, `2048`, and `4096`. Valid values for deployment type `MULTI_AZ_2` and `SINGLE_AZ_2` are `384`,`768`,`1536`,`3072`,`6144` where `haPairs` is `1`. Valid values for deployment type `SINGLE_AZ_2` are `1536`, `3072`, and `6144` where `haPairs` is greater than 1. This parameter is only supported when specifying the haPairs parameter. Either throughputCapacity or throughputCapacityPerHaPair must be specified.
     */
    readonly throughputCapacityPerHaPair: pulumi.Output<number>;
    /**
     * Identifier of the Virtual Private Cloud for the file system.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * The preferred start time (in `d:HH:MM` format) to perform weekly maintenance, in the UTC time zone.
     */
    readonly weeklyMaintenanceStartTime: pulumi.Output<string>;
    /**
     * Create a OntapFileSystem resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OntapFileSystemArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OntapFileSystem resources.
 */
export interface OntapFileSystemState {
    /**
     * Amazon Resource Name of the file system.
     */
    arn?: pulumi.Input<string>;
    /**
     * The number of days to retain automatic backups. Setting this to 0 disables automatic backups. You can retain automatic backups for a maximum of 90 days.
     */
    automaticBackupRetentionDays?: pulumi.Input<number>;
    /**
     * A recurring daily time, in the format HH:MM. HH is the zero-padded hour of the day (0-23), and MM is the zero-padded minute of the hour. For example, 05:00 specifies 5 AM daily. Requires `automaticBackupRetentionDays` to be set.
     */
    dailyAutomaticBackupStartTime?: pulumi.Input<string>;
    /**
     * The filesystem deployment type. Supports `MULTI_AZ_1`, `MULTI_AZ_2`, `SINGLE_AZ_1`, and `SINGLE_AZ_2`.
     */
    deploymentType?: pulumi.Input<string>;
    /**
     * The SSD IOPS configuration for the Amazon FSx for NetApp ONTAP file system. See Disk Iops Configuration below.
     */
    diskIopsConfiguration?: pulumi.Input<inputs.fsx.OntapFileSystemDiskIopsConfiguration>;
    /**
     * The Domain Name Service (DNS) name for the file system. You can mount your file system using its DNS name.
     */
    dnsName?: pulumi.Input<string>;
    /**
     * Specifies the IP address range in which the endpoints to access your file system will be created. By default, Amazon FSx selects an unused IP address range for you from the 198.19.* range.
     */
    endpointIpAddressRange?: pulumi.Input<string>;
    /**
     * The endpoints that are used to access data or to manage the file system using the NetApp ONTAP CLI, REST API, or NetApp SnapMirror. See Endpoints below.
     */
    endpoints?: pulumi.Input<pulumi.Input<inputs.fsx.OntapFileSystemEndpoint>[]>;
    /**
     * The ONTAP administrative password for the fsxadmin user that you can use to administer your file system using the ONTAP CLI and REST API.
     */
    fsxAdminPassword?: pulumi.Input<string>;
    /**
     * The number of haPairs to deploy for the file system. Valid value is 1 for `SINGLE_AZ_1` or `MULTI_AZ_1` and `MULTI_AZ_2`. Valid values are 1 through 12 for `SINGLE_AZ_2`.
     */
    haPairs?: pulumi.Input<number>;
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
     * The ID for a subnet. A subnet is a range of IP addresses in your virtual private cloud (VPC).
     */
    preferredSubnetId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the VPC route tables in which your file system's endpoints will be created. You should specify all VPC route tables associated with the subnets in which your clients are located. By default, Amazon FSx selects your VPC's default route table.
     */
    routeTableIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of IDs for the security groups that apply to the specified network interfaces created for file system access. These security groups will apply to all network interfaces.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The storage capacity (GiB) of the file system. Valid values between `1024` and `196608` for file systems with deploymentType `SINGLE_AZ_1` and `MULTI_AZ_1`. Valid values are between `1024` and `524288` for `MULTI_AZ_2`. Valid values between `1024` (`1024` per ha pair) and `1048576` for file systems with deploymentType `SINGLE_AZ_2`. For `SINGLE_AZ_2`, the `1048576` (1PB) maximum is only supported when using 2 or more ha_pairs, the maximum is `524288` (512TB) when using 1 ha_pair.
     */
    storageCapacity?: pulumi.Input<number>;
    /**
     * The filesystem storage type. defaults to `SSD`.
     */
    storageType?: pulumi.Input<string>;
    /**
     * A list of IDs for the subnets that the file system will be accessible from. Up to 2 subnets can be provided.
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
     * Sets the throughput capacity (in MBps) for the file system that you're creating. Valid values are `128`, `256`, `512`, `1024`, `2048`, and `4096`. This parameter is only supported when not using the haPairs parameter. Either throughputCapacity or throughputCapacityPerHaPair must be specified.
     */
    throughputCapacity?: pulumi.Input<number>;
    /**
     * Sets the per-HA-pair throughput capacity (in MBps) for the file system that you're creating, as opposed to `throughputCapacity` which specifies the total throughput capacity for the file system. Valid value for `MULTI_AZ_1` and `SINGLE_AZ_1` are `128`, `256`, `512`, `1024`, `2048`, and `4096`. Valid values for deployment type `MULTI_AZ_2` and `SINGLE_AZ_2` are `384`,`768`,`1536`,`3072`,`6144` where `haPairs` is `1`. Valid values for deployment type `SINGLE_AZ_2` are `1536`, `3072`, and `6144` where `haPairs` is greater than 1. This parameter is only supported when specifying the haPairs parameter. Either throughputCapacity or throughputCapacityPerHaPair must be specified.
     */
    throughputCapacityPerHaPair?: pulumi.Input<number>;
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
 * The set of arguments for constructing a OntapFileSystem resource.
 */
export interface OntapFileSystemArgs {
    /**
     * The number of days to retain automatic backups. Setting this to 0 disables automatic backups. You can retain automatic backups for a maximum of 90 days.
     */
    automaticBackupRetentionDays?: pulumi.Input<number>;
    /**
     * A recurring daily time, in the format HH:MM. HH is the zero-padded hour of the day (0-23), and MM is the zero-padded minute of the hour. For example, 05:00 specifies 5 AM daily. Requires `automaticBackupRetentionDays` to be set.
     */
    dailyAutomaticBackupStartTime?: pulumi.Input<string>;
    /**
     * The filesystem deployment type. Supports `MULTI_AZ_1`, `MULTI_AZ_2`, `SINGLE_AZ_1`, and `SINGLE_AZ_2`.
     */
    deploymentType: pulumi.Input<string>;
    /**
     * The SSD IOPS configuration for the Amazon FSx for NetApp ONTAP file system. See Disk Iops Configuration below.
     */
    diskIopsConfiguration?: pulumi.Input<inputs.fsx.OntapFileSystemDiskIopsConfiguration>;
    /**
     * Specifies the IP address range in which the endpoints to access your file system will be created. By default, Amazon FSx selects an unused IP address range for you from the 198.19.* range.
     */
    endpointIpAddressRange?: pulumi.Input<string>;
    /**
     * The ONTAP administrative password for the fsxadmin user that you can use to administer your file system using the ONTAP CLI and REST API.
     */
    fsxAdminPassword?: pulumi.Input<string>;
    /**
     * The number of haPairs to deploy for the file system. Valid value is 1 for `SINGLE_AZ_1` or `MULTI_AZ_1` and `MULTI_AZ_2`. Valid values are 1 through 12 for `SINGLE_AZ_2`.
     */
    haPairs?: pulumi.Input<number>;
    /**
     * ARN for the KMS Key to encrypt the file system at rest, Defaults to an AWS managed KMS Key.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The ID for a subnet. A subnet is a range of IP addresses in your virtual private cloud (VPC).
     */
    preferredSubnetId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the VPC route tables in which your file system's endpoints will be created. You should specify all VPC route tables associated with the subnets in which your clients are located. By default, Amazon FSx selects your VPC's default route table.
     */
    routeTableIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of IDs for the security groups that apply to the specified network interfaces created for file system access. These security groups will apply to all network interfaces.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The storage capacity (GiB) of the file system. Valid values between `1024` and `196608` for file systems with deploymentType `SINGLE_AZ_1` and `MULTI_AZ_1`. Valid values are between `1024` and `524288` for `MULTI_AZ_2`. Valid values between `1024` (`1024` per ha pair) and `1048576` for file systems with deploymentType `SINGLE_AZ_2`. For `SINGLE_AZ_2`, the `1048576` (1PB) maximum is only supported when using 2 or more ha_pairs, the maximum is `524288` (512TB) when using 1 ha_pair.
     */
    storageCapacity: pulumi.Input<number>;
    /**
     * The filesystem storage type. defaults to `SSD`.
     */
    storageType?: pulumi.Input<string>;
    /**
     * A list of IDs for the subnets that the file system will be accessible from. Up to 2 subnets can be provided.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the file system. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Sets the throughput capacity (in MBps) for the file system that you're creating. Valid values are `128`, `256`, `512`, `1024`, `2048`, and `4096`. This parameter is only supported when not using the haPairs parameter. Either throughputCapacity or throughputCapacityPerHaPair must be specified.
     */
    throughputCapacity?: pulumi.Input<number>;
    /**
     * Sets the per-HA-pair throughput capacity (in MBps) for the file system that you're creating, as opposed to `throughputCapacity` which specifies the total throughput capacity for the file system. Valid value for `MULTI_AZ_1` and `SINGLE_AZ_1` are `128`, `256`, `512`, `1024`, `2048`, and `4096`. Valid values for deployment type `MULTI_AZ_2` and `SINGLE_AZ_2` are `384`,`768`,`1536`,`3072`,`6144` where `haPairs` is `1`. Valid values for deployment type `SINGLE_AZ_2` are `1536`, `3072`, and `6144` where `haPairs` is greater than 1. This parameter is only supported when specifying the haPairs parameter. Either throughputCapacity or throughputCapacityPerHaPair must be specified.
     */
    throughputCapacityPerHaPair?: pulumi.Input<number>;
    /**
     * The preferred start time (in `d:HH:MM` format) to perform weekly maintenance, in the UTC time zone.
     */
    weeklyMaintenanceStartTime?: pulumi.Input<string>;
}
