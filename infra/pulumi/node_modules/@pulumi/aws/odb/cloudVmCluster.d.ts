import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const withMinimumParameter = new aws.odb.CloudVmCluster("with_minimum_parameter", {
 *     displayName: "my_vm_cluster",
 *     cloudExadataInfrastructureId: "<aws_odb_cloud_exadata_infrastructure_id>",
 *     cpuCoreCount: 6,
 *     giVersion: "23.0.0.0",
 *     hostnamePrefix: "apollo12",
 *     sshPublicKeys: ["public-ssh-key"],
 *     odbNetworkId: "<aws_odb_network_id>",
 *     isLocalBackupEnabled: true,
 *     isSparseDiskgroupEnabled: true,
 *     licenseModel: "LICENSE_INCLUDED",
 *     dataStorageSizeInTbs: 20,
 *     dbServers: [
 *         "db-server-1",
 *         "db-server-2",
 *     ],
 *     dbNodeStorageSizeInGbs: 120,
 *     memorySizeInGbs: 60,
 *     dataCollectionOptions: {
 *         isDiagnosticsEventsEnabled: false,
 *         isHealthMonitoringEnabled: false,
 *         isIncidentLogsEnabled: false,
 *     },
 * });
 * const withAllParameters = new aws.odb.CloudVmCluster("with_all_parameters", {
 *     displayName: "my_vm_cluster",
 *     cloudExadataInfrastructureId: "<aws_odb_cloud_exadata_infrastructure_id>",
 *     cpuCoreCount: 6,
 *     giVersion: "23.0.0.0",
 *     hostnamePrefix: "apollo12",
 *     sshPublicKeys: ["my-ssh-key"],
 *     odbNetworkId: "<aws_odb_network_id>",
 *     isLocalBackupEnabled: true,
 *     isSparseDiskgroupEnabled: true,
 *     licenseModel: "LICENSE_INCLUDED",
 *     dataStorageSizeInTbs: 20,
 *     dbServers: [
 *         "my-dbserver-1",
 *         "my-db-server-2",
 *     ],
 *     dbNodeStorageSizeInGbs: 120,
 *     memorySizeInGbs: 60,
 *     clusterName: "julia-13",
 *     timezone: "UTC",
 *     scanListenerPortTcp: 1521,
 *     tags: {
 *         env: "dev",
 *     },
 *     dataCollectionOptions: {
 *         isDiagnosticsEventsEnabled: true,
 *         isHealthMonitoringEnabled: true,
 *         isIncidentLogsEnabled: true,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import cloud vm cluster using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:odb/cloudVmCluster:CloudVmCluster example example
 * ```
 */
export declare class CloudVmCluster extends pulumi.CustomResource {
    /**
     * Get an existing CloudVmCluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CloudVmClusterState, opts?: pulumi.CustomResourceOptions): CloudVmCluster;
    /**
     * Returns true if the given object is an instance of CloudVmCluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CloudVmCluster;
    /**
     * The Amazon Resource Name (ARN) for the cloud vm cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The unique identifier of the Exadata infrastructure for this VM cluster. Changing this will create a new resource.
     */
    readonly cloudExadataInfrastructureId: pulumi.Output<string>;
    /**
     * The name of the Grid Infrastructure (GI) cluster. Changing this will create a new resource.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * The compute model used when the instance is created or cloned — either ECPU or OCPU. ECPU is a virtualized compute unit; OCPU is a physical processor core with hyper-threading.
     */
    readonly computeModel: pulumi.Output<string>;
    /**
     * The number of CPU cores to enable on the VM cluster. Changing this will create a new resource.
     */
    readonly cpuCoreCount: pulumi.Output<number>;
    /**
     * The timestamp when the VM cluster was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The set of preferences for the various diagnostic collection options for the VM cluster.
     */
    readonly dataCollectionOptions: pulumi.Output<outputs.odb.CloudVmClusterDataCollectionOptions | undefined>;
    /**
     * The size of the data disk group, in terabytes (TBs), to allocate for the VM cluster. Changing this will create a new resource.
     *
     * The following arguments are optional:
     */
    readonly dataStorageSizeInTbs: pulumi.Output<number>;
    /**
     * The amount of local node storage, in gigabytes (GBs), to allocate for the VM cluster. Changing this will create a new resource.
     */
    readonly dbNodeStorageSizeInGbs: pulumi.Output<number>;
    /**
     * The list of database servers for the VM cluster. Changing this will create a new resource.
     */
    readonly dbServers: pulumi.Output<string[]>;
    /**
     * The type of redundancy for the VM cluster: NORMAL (2-way) or HIGH (3-way).
     * * `AttrDomain` - The domain name associated with the VM cluster.
     */
    readonly diskRedundancy: pulumi.Output<string>;
    /**
     * A user-friendly name for the VM cluster. Changing this will create a new resource.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * The domain name associated with the VM cluster.
     */
    readonly domain: pulumi.Output<string>;
    /**
     * A valid software version of Oracle Grid Infrastructure (GI). To get the list of valid values, use the ListGiVersions operation and specify the shape of the Exadata infrastructure. Example: 19.0.0.0 Changing this will create a new resource.
     */
    readonly giVersion: pulumi.Output<string>;
    /**
     * A complete software version of Oracle Grid Infrastructure (GI).
     */
    readonly giVersionComputed: pulumi.Output<string>;
    /**
     * The host name prefix for the VM cluster. Constraints: - Can't be "localhost" or "hostname". - Can't contain "-version". - The maximum length of the combined hostname and domain is 63 characters. - The hostname must be unique within the subnet. Changing this will create a new resource.
     */
    readonly hostnamePrefix: pulumi.Output<string>;
    /**
     * The host name for the VM cluster. Constraints: - Can't be "localhost" or "hostname". - Can't contain "-version". - The maximum length of the combined hostname and domain is 63 characters. - The hostname must be unique within the subnet. This member is required. Changing this will create a new resource.
     */
    readonly hostnamePrefixComputed: pulumi.Output<string>;
    /**
     * The Exadata IORM (I/O Resource Manager) configuration cache details for the VM cluster.
     */
    readonly iormConfigCaches: pulumi.Output<outputs.odb.CloudVmClusterIormConfigCach[]>;
    /**
     * Specifies whether to enable database backups to local Exadata storage for the VM cluster. Changing this will create a new resource.
     */
    readonly isLocalBackupEnabled: pulumi.Output<boolean>;
    /**
     * Specifies whether to create a sparse disk group for the VM cluster. Changing this will create a new resource.
     */
    readonly isSparseDiskgroupEnabled: pulumi.Output<boolean>;
    /**
     * The OCID of the most recent maintenance update history entry.
     */
    readonly lastUpdateHistoryEntryId: pulumi.Output<string>;
    /**
     * The Oracle license model to apply to the VM cluster. Default: LICENSE_INCLUDED. Changing this will create a new resource.
     */
    readonly licenseModel: pulumi.Output<string>;
    /**
     * The listener port number configured on the VM cluster.
     */
    readonly listenerPort: pulumi.Output<number>;
    /**
     * The amount of memory, in gigabytes (GBs), to allocate for the VM cluster. Changing this will create a new resource.
     */
    readonly memorySizeInGbs: pulumi.Output<number>;
    /**
     * The total number of nodes in the VM cluster.
     */
    readonly nodeCount: pulumi.Output<number>;
    /**
     * The name of the OCI resource anchor associated with the VM cluster.
     */
    readonly ociResourceAnchorName: pulumi.Output<string>;
    /**
     * The HTTPS link to the VM cluster resource in OCI.
     */
    readonly ociUrl: pulumi.Output<string>;
    /**
     * The OCID (Oracle Cloud Identifier) of the VM cluster.
     */
    readonly ocid: pulumi.Output<string>;
    /**
     * The unique identifier of the ODB network for the VM cluster. Changing this will create a new resource.
     */
    readonly odbNetworkId: pulumi.Output<string>;
    /**
     * The percentage of progress made on the current operation for the VM cluster.
     */
    readonly percentProgress: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The fully qualified domain name (FQDN) for the SCAN IP addresses associated with the VM cluster.
     */
    readonly scanDnsName: pulumi.Output<string>;
    /**
     * The OCID of the DNS record for the SCAN IPs linked to the VM cluster.
     */
    readonly scanDnsRecordId: pulumi.Output<string>;
    /**
     * The list of OCIDs for SCAN IP addresses associated with the VM cluster.
     */
    readonly scanIpIds: pulumi.Output<string[]>;
    /**
     * The port number for TCP connections to the single client access name (SCAN) listener. Valid values: 1024–8999, except 2484, 6100, 6200, 7060, 7070, 7085, and 7879. Default: 1521. Changing this will create a new resource.
     */
    readonly scanListenerPortTcp: pulumi.Output<number>;
    /**
     * The hardware model name of the Exadata infrastructure running the VM cluster.
     */
    readonly shape: pulumi.Output<string>;
    /**
     * The public key portion of one or more key pairs used for SSH access to the VM cluster. Changing this will create a new resource.
     */
    readonly sshPublicKeys: pulumi.Output<string[]>;
    /**
     * The current lifecycle status of the VM cluster.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Additional information regarding the current status of the VM cluster.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * The local node storage allocated to the VM cluster, in gigabytes (GB).
     */
    readonly storageSizeInGbs: pulumi.Output<number>;
    /**
     * The operating system version of the image chosen for the VM cluster.
     */
    readonly systemVersion: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.odb.CloudVmClusterTimeouts | undefined>;
    /**
     * The configured time zone of the VM cluster. Changing this will create a new resource.
     */
    readonly timezone: pulumi.Output<string>;
    /**
     * The virtual IP (VIP) addresses assigned to the VM cluster. CRS assigns one VIP per node for failover support.
     */
    readonly vipIds: pulumi.Output<string[]>;
    /**
     * Create a CloudVmCluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CloudVmClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CloudVmCluster resources.
 */
export interface CloudVmClusterState {
    /**
     * The Amazon Resource Name (ARN) for the cloud vm cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * The unique identifier of the Exadata infrastructure for this VM cluster. Changing this will create a new resource.
     */
    cloudExadataInfrastructureId?: pulumi.Input<string>;
    /**
     * The name of the Grid Infrastructure (GI) cluster. Changing this will create a new resource.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * The compute model used when the instance is created or cloned — either ECPU or OCPU. ECPU is a virtualized compute unit; OCPU is a physical processor core with hyper-threading.
     */
    computeModel?: pulumi.Input<string>;
    /**
     * The number of CPU cores to enable on the VM cluster. Changing this will create a new resource.
     */
    cpuCoreCount?: pulumi.Input<number>;
    /**
     * The timestamp when the VM cluster was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The set of preferences for the various diagnostic collection options for the VM cluster.
     */
    dataCollectionOptions?: pulumi.Input<inputs.odb.CloudVmClusterDataCollectionOptions>;
    /**
     * The size of the data disk group, in terabytes (TBs), to allocate for the VM cluster. Changing this will create a new resource.
     *
     * The following arguments are optional:
     */
    dataStorageSizeInTbs?: pulumi.Input<number>;
    /**
     * The amount of local node storage, in gigabytes (GBs), to allocate for the VM cluster. Changing this will create a new resource.
     */
    dbNodeStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The list of database servers for the VM cluster. Changing this will create a new resource.
     */
    dbServers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of redundancy for the VM cluster: NORMAL (2-way) or HIGH (3-way).
     * * `AttrDomain` - The domain name associated with the VM cluster.
     */
    diskRedundancy?: pulumi.Input<string>;
    /**
     * A user-friendly name for the VM cluster. Changing this will create a new resource.
     */
    displayName?: pulumi.Input<string>;
    /**
     * The domain name associated with the VM cluster.
     */
    domain?: pulumi.Input<string>;
    /**
     * A valid software version of Oracle Grid Infrastructure (GI). To get the list of valid values, use the ListGiVersions operation and specify the shape of the Exadata infrastructure. Example: 19.0.0.0 Changing this will create a new resource.
     */
    giVersion?: pulumi.Input<string>;
    /**
     * A complete software version of Oracle Grid Infrastructure (GI).
     */
    giVersionComputed?: pulumi.Input<string>;
    /**
     * The host name prefix for the VM cluster. Constraints: - Can't be "localhost" or "hostname". - Can't contain "-version". - The maximum length of the combined hostname and domain is 63 characters. - The hostname must be unique within the subnet. Changing this will create a new resource.
     */
    hostnamePrefix?: pulumi.Input<string>;
    /**
     * The host name for the VM cluster. Constraints: - Can't be "localhost" or "hostname". - Can't contain "-version". - The maximum length of the combined hostname and domain is 63 characters. - The hostname must be unique within the subnet. This member is required. Changing this will create a new resource.
     */
    hostnamePrefixComputed?: pulumi.Input<string>;
    /**
     * The Exadata IORM (I/O Resource Manager) configuration cache details for the VM cluster.
     */
    iormConfigCaches?: pulumi.Input<pulumi.Input<inputs.odb.CloudVmClusterIormConfigCach>[]>;
    /**
     * Specifies whether to enable database backups to local Exadata storage for the VM cluster. Changing this will create a new resource.
     */
    isLocalBackupEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether to create a sparse disk group for the VM cluster. Changing this will create a new resource.
     */
    isSparseDiskgroupEnabled?: pulumi.Input<boolean>;
    /**
     * The OCID of the most recent maintenance update history entry.
     */
    lastUpdateHistoryEntryId?: pulumi.Input<string>;
    /**
     * The Oracle license model to apply to the VM cluster. Default: LICENSE_INCLUDED. Changing this will create a new resource.
     */
    licenseModel?: pulumi.Input<string>;
    /**
     * The listener port number configured on the VM cluster.
     */
    listenerPort?: pulumi.Input<number>;
    /**
     * The amount of memory, in gigabytes (GBs), to allocate for the VM cluster. Changing this will create a new resource.
     */
    memorySizeInGbs?: pulumi.Input<number>;
    /**
     * The total number of nodes in the VM cluster.
     */
    nodeCount?: pulumi.Input<number>;
    /**
     * The name of the OCI resource anchor associated with the VM cluster.
     */
    ociResourceAnchorName?: pulumi.Input<string>;
    /**
     * The HTTPS link to the VM cluster resource in OCI.
     */
    ociUrl?: pulumi.Input<string>;
    /**
     * The OCID (Oracle Cloud Identifier) of the VM cluster.
     */
    ocid?: pulumi.Input<string>;
    /**
     * The unique identifier of the ODB network for the VM cluster. Changing this will create a new resource.
     */
    odbNetworkId?: pulumi.Input<string>;
    /**
     * The percentage of progress made on the current operation for the VM cluster.
     */
    percentProgress?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The fully qualified domain name (FQDN) for the SCAN IP addresses associated with the VM cluster.
     */
    scanDnsName?: pulumi.Input<string>;
    /**
     * The OCID of the DNS record for the SCAN IPs linked to the VM cluster.
     */
    scanDnsRecordId?: pulumi.Input<string>;
    /**
     * The list of OCIDs for SCAN IP addresses associated with the VM cluster.
     */
    scanIpIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The port number for TCP connections to the single client access name (SCAN) listener. Valid values: 1024–8999, except 2484, 6100, 6200, 7060, 7070, 7085, and 7879. Default: 1521. Changing this will create a new resource.
     */
    scanListenerPortTcp?: pulumi.Input<number>;
    /**
     * The hardware model name of the Exadata infrastructure running the VM cluster.
     */
    shape?: pulumi.Input<string>;
    /**
     * The public key portion of one or more key pairs used for SSH access to the VM cluster. Changing this will create a new resource.
     */
    sshPublicKeys?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The current lifecycle status of the VM cluster.
     */
    status?: pulumi.Input<string>;
    /**
     * Additional information regarding the current status of the VM cluster.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * The local node storage allocated to the VM cluster, in gigabytes (GB).
     */
    storageSizeInGbs?: pulumi.Input<number>;
    /**
     * The operating system version of the image chosen for the VM cluster.
     */
    systemVersion?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.odb.CloudVmClusterTimeouts>;
    /**
     * The configured time zone of the VM cluster. Changing this will create a new resource.
     */
    timezone?: pulumi.Input<string>;
    /**
     * The virtual IP (VIP) addresses assigned to the VM cluster. CRS assigns one VIP per node for failover support.
     */
    vipIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a CloudVmCluster resource.
 */
export interface CloudVmClusterArgs {
    /**
     * The unique identifier of the Exadata infrastructure for this VM cluster. Changing this will create a new resource.
     */
    cloudExadataInfrastructureId: pulumi.Input<string>;
    /**
     * The name of the Grid Infrastructure (GI) cluster. Changing this will create a new resource.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * The number of CPU cores to enable on the VM cluster. Changing this will create a new resource.
     */
    cpuCoreCount: pulumi.Input<number>;
    /**
     * The set of preferences for the various diagnostic collection options for the VM cluster.
     */
    dataCollectionOptions?: pulumi.Input<inputs.odb.CloudVmClusterDataCollectionOptions>;
    /**
     * The size of the data disk group, in terabytes (TBs), to allocate for the VM cluster. Changing this will create a new resource.
     *
     * The following arguments are optional:
     */
    dataStorageSizeInTbs: pulumi.Input<number>;
    /**
     * The amount of local node storage, in gigabytes (GBs), to allocate for the VM cluster. Changing this will create a new resource.
     */
    dbNodeStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The list of database servers for the VM cluster. Changing this will create a new resource.
     */
    dbServers: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A user-friendly name for the VM cluster. Changing this will create a new resource.
     */
    displayName: pulumi.Input<string>;
    /**
     * A valid software version of Oracle Grid Infrastructure (GI). To get the list of valid values, use the ListGiVersions operation and specify the shape of the Exadata infrastructure. Example: 19.0.0.0 Changing this will create a new resource.
     */
    giVersion: pulumi.Input<string>;
    /**
     * The host name prefix for the VM cluster. Constraints: - Can't be "localhost" or "hostname". - Can't contain "-version". - The maximum length of the combined hostname and domain is 63 characters. - The hostname must be unique within the subnet. Changing this will create a new resource.
     */
    hostnamePrefix: pulumi.Input<string>;
    /**
     * Specifies whether to enable database backups to local Exadata storage for the VM cluster. Changing this will create a new resource.
     */
    isLocalBackupEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether to create a sparse disk group for the VM cluster. Changing this will create a new resource.
     */
    isSparseDiskgroupEnabled?: pulumi.Input<boolean>;
    /**
     * The Oracle license model to apply to the VM cluster. Default: LICENSE_INCLUDED. Changing this will create a new resource.
     */
    licenseModel?: pulumi.Input<string>;
    /**
     * The amount of memory, in gigabytes (GBs), to allocate for the VM cluster. Changing this will create a new resource.
     */
    memorySizeInGbs?: pulumi.Input<number>;
    /**
     * The unique identifier of the ODB network for the VM cluster. Changing this will create a new resource.
     */
    odbNetworkId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The port number for TCP connections to the single client access name (SCAN) listener. Valid values: 1024–8999, except 2484, 6100, 6200, 7060, 7070, 7085, and 7879. Default: 1521. Changing this will create a new resource.
     */
    scanListenerPortTcp?: pulumi.Input<number>;
    /**
     * The public key portion of one or more key pairs used for SSH access to the VM cluster. Changing this will create a new resource.
     */
    sshPublicKeys: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.CloudVmClusterTimeouts>;
    /**
     * The configured time zone of the VM cluster. Changing this will create a new resource.
     */
    timezone?: pulumi.Input<string>;
}
