import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource managing cloud autonomous vm cluster in AWS for Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const avmcWithMinimumParameters = new aws.odb.CloudAutonomousVmCluster("avmc_with_minimum_parameters", {
 *     cloudExadataInfrastructureId: "<aws_odb_cloud_exadata_infrastructure_id>",
 *     odbNetworkId: "<aws_odb_network_id>",
 *     displayName: "my_autonomous_vm_cluster",
 *     autonomousDataStorageSizeInTbs: 5,
 *     memoryPerOracleComputeUnitInGbs: 2,
 *     totalContainerDatabases: 1,
 *     cpuCoreCountPerNode: 40,
 *     licenseModel: "LICENSE_INCLUDED",
 *     dbServers: ["<my_db_server_id>"],
 *     scanListenerPortTls: 8561,
 *     scanListenerPortNonTls: 1024,
 *     maintenanceWindow: {
 *         preference: "NO_PREFERENCE",
 *     },
 * });
 * const avmcWithAllParams = new aws.odb.CloudAutonomousVmCluster("avmc_with_all_params", {
 *     description: "my first avmc",
 *     timeZone: "UTC",
 *     cloudExadataInfrastructureId: "<aws_odb_cloud_exadata_infrastructure_id>",
 *     odbNetworkId: "<aws_odb_network_id>",
 *     displayName: "my_autonomous_vm_cluster",
 *     autonomousDataStorageSizeInTbs: 5,
 *     memoryPerOracleComputeUnitInGbs: 2,
 *     totalContainerDatabases: 1,
 *     cpuCoreCountPerNode: 40,
 *     licenseModel: "LICENSE_INCLUDED",
 *     dbServers: [
 *         "<my_db_server_1>",
 *         "<my_db_server_2>",
 *     ],
 *     scanListenerPortTls: 8561,
 *     scanListenerPortNonTls: 1024,
 *     maintenanceWindow: {
 *         daysOfWeeks: [
 *             {
 *                 name: "MONDAY",
 *             },
 *             {
 *                 name: "TUESDAY",
 *             },
 *         ],
 *         hoursOfDays: [
 *             4,
 *             16,
 *         ],
 *         leadTimeInWeeks: 3,
 *         months: [
 *             {
 *                 name: "FEBRUARY",
 *             },
 *             {
 *                 name: "MAY",
 *             },
 *             {
 *                 name: "AUGUST",
 *             },
 *             {
 *                 name: "NOVEMBER",
 *             },
 *         ],
 *         preference: "CUSTOM_PREFERENCE",
 *         weeksOfMonths: [
 *             2,
 *             4,
 *         ],
 *     },
 *     tags: {
 *         env: "dev",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import cloud autonomous vm cluster `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:odb/cloudAutonomousVmCluster:CloudAutonomousVmCluster example example
 * ```
 */
export declare class CloudAutonomousVmCluster extends pulumi.CustomResource {
    /**
     * Get an existing CloudAutonomousVmCluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CloudAutonomousVmClusterState, opts?: pulumi.CustomResourceOptions): CloudAutonomousVmCluster;
    /**
     * Returns true if the given object is an instance of CloudAutonomousVmCluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CloudAutonomousVmCluster;
    /**
     * The Amazon Resource Name (ARN) for the Exadata infrastructure.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The progress of the current operation on the Autonomous VM cluster, as a percentage.
     */
    readonly autonomousDataStoragePercentage: pulumi.Output<number>;
    readonly autonomousDataStorageSizeInTbs: pulumi.Output<number>;
    /**
     * The available data storage space for Autonomous Databases in the Autonomous VM cluster, in TB.
     */
    readonly availableAutonomousDataStorageSizeInTbs: pulumi.Output<number>;
    /**
     * The number of Autonomous CDBs that you can create with the currently available storage.
     */
    readonly availableContainerDatabases: pulumi.Output<number>;
    /**
     * The number of CPU cores available for allocation to Autonomous Databases.
     */
    readonly availableCpus: pulumi.Output<number>;
    readonly cloudExadataInfrastructureId: pulumi.Output<string>;
    /**
     * The compute model of the Autonomous VM cluster: ECPU or OCPU.
     */
    readonly computeModel: pulumi.Output<string>;
    /**
     * The total number of CPU cores in the Autonomous VM cluster.
     */
    readonly cpuCoreCount: pulumi.Output<number>;
    /**
     * The number of CPU cores enabled per node in the Autonomous VM cluster.
     */
    readonly cpuCoreCountPerNode: pulumi.Output<number>;
    /**
     * The percentage of total CPU cores currently in use in the Autonomous VM cluster.
     */
    readonly cpuPercentage: pulumi.Output<number>;
    /**
     * The date and time when the Autonomous VM cluster was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The total data storage allocated to the Autonomous VM cluster, in GB.
     */
    readonly dataStorageSizeInGbs: pulumi.Output<number>;
    /**
     * The total data storage allocated to the Autonomous VM cluster, in TB.
     */
    readonly dataStorageSizeInTbs: pulumi.Output<number>;
    readonly dbServers: pulumi.Output<string[]>;
    /**
     * The description of the Autonomous VM cluster.
     */
    readonly description: pulumi.Output<string | undefined>;
    readonly displayName: pulumi.Output<string>;
    /**
     * The domain name of the Autonomous VM cluster.
     */
    readonly domain: pulumi.Output<string>;
    /**
     * The minimum value to which you can scale down the Exadata storage, in TB.
     */
    readonly exadataStorageInTbsLowestScaledValue: pulumi.Output<number>;
    /**
     * The hostname of the Autonomous VM cluster.
     */
    readonly hostname: pulumi.Output<string>;
    readonly isMtlsEnabledVmCluster: pulumi.Output<boolean>;
    readonly licenseModel: pulumi.Output<string>;
    /**
     * The maintenance window of the Autonomous VM cluster.
     */
    readonly maintenanceWindow: pulumi.Output<outputs.odb.CloudAutonomousVmClusterMaintenanceWindow | undefined>;
    /**
     * The minimum value to which you can scale down the maximum number of Autonomous CDBs.
     */
    readonly maxAcdsLowestScaledValue: pulumi.Output<number>;
    readonly memoryPerOracleComputeUnitInGbs: pulumi.Output<number>;
    /**
     * The total amount of memory allocated to the Autonomous VM cluster, in gigabytes(GB).
     */
    readonly memorySizeInGbs: pulumi.Output<number>;
    /**
     * The number of database server nodes in the Autonomous VM cluster.
     */
    readonly nodeCount: pulumi.Output<number>;
    /**
     * The number of Autonomous CDBs that can't be provisioned because of resource constraints.
     */
    readonly nonProvisionableAutonomousContainerDatabases: pulumi.Output<number>;
    /**
     * The name of the OCI resource anchor associated with this Autonomous VM cluster.
     */
    readonly ociResourceAnchorName: pulumi.Output<string>;
    /**
     * The URL for accessing the OCI console page for this Autonomous VM cluster.
     */
    readonly ociUrl: pulumi.Output<string>;
    /**
     * The Oracle Cloud Identifier (OCID) of the Autonomous VM cluster.
     */
    readonly ocid: pulumi.Output<string>;
    readonly odbNetworkId: pulumi.Output<string>;
    /**
     * The local node storage allocated to the Autonomous VM cluster, in gigabytes (GB).
     */
    readonly odbNodeStorageSizeInGbs: pulumi.Output<number>;
    /**
     * The progress of the current operation on the Autonomous VM cluster, as a percentage.
     */
    readonly percentProgress: pulumi.Output<number>;
    /**
     * The number of Autonomous CDBs that can be provisioned in the Autonomous VM cluster.
     */
    readonly provisionableAutonomousContainerDatabases: pulumi.Output<number>;
    /**
     * The number of Autonomous CDBs currently provisioned in the Autonomous VM cluster.
     */
    readonly provisionedAutonomousContainerDatabases: pulumi.Output<number>;
    /**
     * The number of CPUs provisioned in the Autonomous VM cluster.
     */
    readonly provisionedCpus: pulumi.Output<number>;
    /**
     * The number of CPU cores that can be reclaimed from terminated or scaled-down Autonomous Databases.
     */
    readonly reclaimableCpus: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The number of CPU cores reserved for system operations and redundancy.
     */
    readonly reservedCpus: pulumi.Output<number>;
    readonly scanListenerPortNonTls: pulumi.Output<number>;
    readonly scanListenerPortTls: pulumi.Output<number>;
    /**
     * The shape of the Exadata infrastructure for the Autonomous VM cluster.
     */
    readonly shape: pulumi.Output<string>;
    /**
     * The status of the Autonomous VM cluster. Possible values include CREATING, AVAILABLE, UPDATING, DELETING, DELETED, FAILED.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Additional information about the current status of the Autonomous VM cluster.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The combined set of user-defined and provider-defined tags.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The expiration date and time of the database SSL certificate.
     */
    readonly timeDatabaseSslCertificateExpires: pulumi.Output<string>;
    /**
     * The expiration date and time of the ORDS certificate.
     */
    readonly timeOrdsCertificateExpires: pulumi.Output<string>;
    readonly timeZone: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.odb.CloudAutonomousVmClusterTimeouts | undefined>;
    readonly totalContainerDatabases: pulumi.Output<number>;
    /**
     * Create a CloudAutonomousVmCluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CloudAutonomousVmClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CloudAutonomousVmCluster resources.
 */
export interface CloudAutonomousVmClusterState {
    /**
     * The Amazon Resource Name (ARN) for the Exadata infrastructure.
     */
    arn?: pulumi.Input<string>;
    /**
     * The progress of the current operation on the Autonomous VM cluster, as a percentage.
     */
    autonomousDataStoragePercentage?: pulumi.Input<number>;
    autonomousDataStorageSizeInTbs?: pulumi.Input<number>;
    /**
     * The available data storage space for Autonomous Databases in the Autonomous VM cluster, in TB.
     */
    availableAutonomousDataStorageSizeInTbs?: pulumi.Input<number>;
    /**
     * The number of Autonomous CDBs that you can create with the currently available storage.
     */
    availableContainerDatabases?: pulumi.Input<number>;
    /**
     * The number of CPU cores available for allocation to Autonomous Databases.
     */
    availableCpus?: pulumi.Input<number>;
    cloudExadataInfrastructureId?: pulumi.Input<string>;
    /**
     * The compute model of the Autonomous VM cluster: ECPU or OCPU.
     */
    computeModel?: pulumi.Input<string>;
    /**
     * The total number of CPU cores in the Autonomous VM cluster.
     */
    cpuCoreCount?: pulumi.Input<number>;
    /**
     * The number of CPU cores enabled per node in the Autonomous VM cluster.
     */
    cpuCoreCountPerNode?: pulumi.Input<number>;
    /**
     * The percentage of total CPU cores currently in use in the Autonomous VM cluster.
     */
    cpuPercentage?: pulumi.Input<number>;
    /**
     * The date and time when the Autonomous VM cluster was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The total data storage allocated to the Autonomous VM cluster, in GB.
     */
    dataStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The total data storage allocated to the Autonomous VM cluster, in TB.
     */
    dataStorageSizeInTbs?: pulumi.Input<number>;
    dbServers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The description of the Autonomous VM cluster.
     */
    description?: pulumi.Input<string>;
    displayName?: pulumi.Input<string>;
    /**
     * The domain name of the Autonomous VM cluster.
     */
    domain?: pulumi.Input<string>;
    /**
     * The minimum value to which you can scale down the Exadata storage, in TB.
     */
    exadataStorageInTbsLowestScaledValue?: pulumi.Input<number>;
    /**
     * The hostname of the Autonomous VM cluster.
     */
    hostname?: pulumi.Input<string>;
    isMtlsEnabledVmCluster?: pulumi.Input<boolean>;
    licenseModel?: pulumi.Input<string>;
    /**
     * The maintenance window of the Autonomous VM cluster.
     */
    maintenanceWindow?: pulumi.Input<inputs.odb.CloudAutonomousVmClusterMaintenanceWindow>;
    /**
     * The minimum value to which you can scale down the maximum number of Autonomous CDBs.
     */
    maxAcdsLowestScaledValue?: pulumi.Input<number>;
    memoryPerOracleComputeUnitInGbs?: pulumi.Input<number>;
    /**
     * The total amount of memory allocated to the Autonomous VM cluster, in gigabytes(GB).
     */
    memorySizeInGbs?: pulumi.Input<number>;
    /**
     * The number of database server nodes in the Autonomous VM cluster.
     */
    nodeCount?: pulumi.Input<number>;
    /**
     * The number of Autonomous CDBs that can't be provisioned because of resource constraints.
     */
    nonProvisionableAutonomousContainerDatabases?: pulumi.Input<number>;
    /**
     * The name of the OCI resource anchor associated with this Autonomous VM cluster.
     */
    ociResourceAnchorName?: pulumi.Input<string>;
    /**
     * The URL for accessing the OCI console page for this Autonomous VM cluster.
     */
    ociUrl?: pulumi.Input<string>;
    /**
     * The Oracle Cloud Identifier (OCID) of the Autonomous VM cluster.
     */
    ocid?: pulumi.Input<string>;
    odbNetworkId?: pulumi.Input<string>;
    /**
     * The local node storage allocated to the Autonomous VM cluster, in gigabytes (GB).
     */
    odbNodeStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The progress of the current operation on the Autonomous VM cluster, as a percentage.
     */
    percentProgress?: pulumi.Input<number>;
    /**
     * The number of Autonomous CDBs that can be provisioned in the Autonomous VM cluster.
     */
    provisionableAutonomousContainerDatabases?: pulumi.Input<number>;
    /**
     * The number of Autonomous CDBs currently provisioned in the Autonomous VM cluster.
     */
    provisionedAutonomousContainerDatabases?: pulumi.Input<number>;
    /**
     * The number of CPUs provisioned in the Autonomous VM cluster.
     */
    provisionedCpus?: pulumi.Input<number>;
    /**
     * The number of CPU cores that can be reclaimed from terminated or scaled-down Autonomous Databases.
     */
    reclaimableCpus?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The number of CPU cores reserved for system operations and redundancy.
     */
    reservedCpus?: pulumi.Input<number>;
    scanListenerPortNonTls?: pulumi.Input<number>;
    scanListenerPortTls?: pulumi.Input<number>;
    /**
     * The shape of the Exadata infrastructure for the Autonomous VM cluster.
     */
    shape?: pulumi.Input<string>;
    /**
     * The status of the Autonomous VM cluster. Possible values include CREATING, AVAILABLE, UPDATING, DELETING, DELETED, FAILED.
     */
    status?: pulumi.Input<string>;
    /**
     * Additional information about the current status of the Autonomous VM cluster.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The combined set of user-defined and provider-defined tags.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The expiration date and time of the database SSL certificate.
     */
    timeDatabaseSslCertificateExpires?: pulumi.Input<string>;
    /**
     * The expiration date and time of the ORDS certificate.
     */
    timeOrdsCertificateExpires?: pulumi.Input<string>;
    timeZone?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.odb.CloudAutonomousVmClusterTimeouts>;
    totalContainerDatabases?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a CloudAutonomousVmCluster resource.
 */
export interface CloudAutonomousVmClusterArgs {
    autonomousDataStorageSizeInTbs: pulumi.Input<number>;
    cloudExadataInfrastructureId: pulumi.Input<string>;
    /**
     * The number of CPU cores enabled per node in the Autonomous VM cluster.
     */
    cpuCoreCountPerNode: pulumi.Input<number>;
    dbServers: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The description of the Autonomous VM cluster.
     */
    description?: pulumi.Input<string>;
    displayName: pulumi.Input<string>;
    isMtlsEnabledVmCluster?: pulumi.Input<boolean>;
    licenseModel?: pulumi.Input<string>;
    /**
     * The maintenance window of the Autonomous VM cluster.
     */
    maintenanceWindow?: pulumi.Input<inputs.odb.CloudAutonomousVmClusterMaintenanceWindow>;
    memoryPerOracleComputeUnitInGbs: pulumi.Input<number>;
    odbNetworkId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    scanListenerPortNonTls: pulumi.Input<number>;
    scanListenerPortTls: pulumi.Input<number>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeZone?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.odb.CloudAutonomousVmClusterTimeouts>;
    totalContainerDatabases: pulumi.Input<number>;
}
