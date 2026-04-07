import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS FinSpace Kx Cluster.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.finspace.KxCluster("example", {
 *     name: "my-tf-kx-cluster",
 *     environmentId: exampleAwsFinspaceKxEnvironment.id,
 *     type: "HDB",
 *     releaseLabel: "1.0",
 *     azMode: "SINGLE",
 *     availabilityZoneId: "use1-az2",
 *     capacityConfiguration: {
 *         nodeType: "kx.s.2xlarge",
 *         nodeCount: 2,
 *     },
 *     vpcConfiguration: {
 *         vpcId: test.id,
 *         securityGroupIds: [exampleAwsSecurityGroup.id],
 *         subnetIds: [exampleAwsSubnet.id],
 *         ipAddressType: "IP_V4",
 *     },
 *     cacheStorageConfigurations: [{
 *         type: "CACHE_1000",
 *         size: 1200,
 *     }],
 *     databases: [{
 *         databaseName: exampleAwsFinspaceKxDatabase.name,
 *         cacheConfiguration: [{
 *             cacheType: "CACHE_1000",
 *             dbPaths: "/",
 *         }],
 *     }],
 *     code: {
 *         s3Bucket: testAwsS3Bucket.id,
 *         s3Key: object.key,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an AWS FinSpace Kx Cluster using the `id` (environment ID and cluster name, comma-delimited). For example:
 *
 * ```sh
 * $ pulumi import aws:finspace/kxCluster:KxCluster example n3ceo7wqxoxcti5tujqwzs,my-tf-kx-cluster
 * ```
 */
export declare class KxCluster extends pulumi.CustomResource {
    /**
     * Get an existing KxCluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KxClusterState, opts?: pulumi.CustomResourceOptions): KxCluster;
    /**
     * Returns true if the given object is an instance of KxCluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KxCluster;
    /**
     * Amazon Resource Name (ARN) identifier of the KX cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration based on which FinSpace will scale in or scale out nodes in your cluster. See auto_scaling_configuration.
     */
    readonly autoScalingConfiguration: pulumi.Output<outputs.finspace.KxClusterAutoScalingConfiguration | undefined>;
    /**
     * The availability zone identifiers for the requested regions. Required when `azMode` is set to SINGLE.
     */
    readonly availabilityZoneId: pulumi.Output<string | undefined>;
    /**
     * The number of availability zones you want to assign per cluster. This can be one of the following:
     * * SINGLE - Assigns one availability zone per cluster.
     * * MULTI - Assigns all the availability zones per cluster.
     */
    readonly azMode: pulumi.Output<string>;
    /**
     * Configurations for a read only cache storage associated with a cluster. This cache will be stored as an FSx Lustre that reads from the S3 store. See cache_storage_configuration.
     */
    readonly cacheStorageConfigurations: pulumi.Output<outputs.finspace.KxClusterCacheStorageConfiguration[] | undefined>;
    /**
     * Structure for the metadata of a cluster. Includes information like the CPUs needed, memory of instances, and number of instances. See capacity_configuration.
     */
    readonly capacityConfiguration: pulumi.Output<outputs.finspace.KxClusterCapacityConfiguration | undefined>;
    /**
     * Details of the custom code that you want to use inside a cluster when analyzing data. Consists of the S3 source bucket, location, object version, and the relative path from where the custom code is loaded into the cluster. See code.
     */
    readonly code: pulumi.Output<outputs.finspace.KxClusterCode | undefined>;
    /**
     * List of key-value pairs to make available inside the cluster.
     */
    readonly commandLineArguments: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Timestamp at which the cluster is created in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly createdTimestamp: pulumi.Output<string>;
    /**
     * KX database that will be available for querying. Defined below.
     */
    readonly databases: pulumi.Output<outputs.finspace.KxClusterDatabase[] | undefined>;
    /**
     * Description of the cluster.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Unique identifier for the KX environment.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * An IAM role that defines a set of permissions associated with a cluster. These permissions are assumed when a cluster attempts to access another cluster.
     */
    readonly executionRole: pulumi.Output<string | undefined>;
    /**
     * Path to Q program that will be run at launch of a cluster. This is a relative path within .zip file that contains the custom code, which will be loaded on the cluster. It must include the file name itself. For example, somedir/init.q.
     */
    readonly initializationScript: pulumi.Output<string | undefined>;
    /**
     * Last timestamp at which the cluster was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly lastModifiedTimestamp: pulumi.Output<string>;
    /**
     * Unique name for the cluster that you want to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Version of FinSpace Managed kdb to run.
     */
    readonly releaseLabel: pulumi.Output<string>;
    /**
     * Size and type of the temporary storage that is used to hold data during the savedown process. This parameter is required when you choose `type` as RDB. All the data written to this storage space is lost when the cluster node is restarted. See savedown_storage_configuration.
     */
    readonly savedownStorageConfiguration: pulumi.Output<outputs.finspace.KxClusterSavedownStorageConfiguration | undefined>;
    /**
     * The structure that stores the configuration details of a scaling group.
     */
    readonly scalingGroupConfiguration: pulumi.Output<outputs.finspace.KxClusterScalingGroupConfiguration | undefined>;
    readonly status: pulumi.Output<string>;
    readonly statusReason: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * A configuration to store Tickerplant logs. It consists of a list of volumes that will be mounted to your cluster. For the cluster type Tickerplant , the location of the TP volume on the cluster will be available by using the global variable .aws.tp_log_path.
     */
    readonly tickerplantLogConfigurations: pulumi.Output<outputs.finspace.KxClusterTickerplantLogConfiguration[] | undefined>;
    /**
     * Type of KDB database. The following types are available:
     * * HDB - Historical Database. The data is only accessible with read-only permissions from one of the FinSpace managed KX databases mounted to the cluster.
     * * RDB - Realtime Database. This type of database captures all the data from a ticker plant and stores it in memory until the end of day, after which it writes all of its data to a disk and reloads the HDB. This cluster type requires local storage for temporary storage of data during the savedown process. If you specify this field in your request, you must provide the `savedownStorageConfiguration` parameter.
     * * GATEWAY - A gateway cluster allows you to access data across processes in kdb systems. It allows you to create your own routing logic using the initialization scripts and custom code. This type of cluster does not require a  writable local storage.
     * * GP - A general purpose cluster allows you to quickly iterate on code during development by granting greater access to system commands and enabling a fast reload of custom code. This cluster type can optionally mount databases including cache and savedown storage. For this cluster type, the node count is fixed at 1. It does not support autoscaling and supports only `SINGLE` AZ mode.
     * * Tickerplant - A tickerplant cluster allows you to subscribe to feed handlers based on IAM permissions. It can publish to RDBs, other Tickerplants, and real-time subscribers (RTS). Tickerplants can persist messages to log, which is readable by any RDB environment. It supports only single-node that is only one kdb process.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Configuration details about the network where the Privatelink endpoint of the cluster resides. See vpc_configuration.
     *
     * The following arguments are optional:
     */
    readonly vpcConfiguration: pulumi.Output<outputs.finspace.KxClusterVpcConfiguration>;
    /**
     * Create a KxCluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KxClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KxCluster resources.
 */
export interface KxClusterState {
    /**
     * Amazon Resource Name (ARN) identifier of the KX cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration based on which FinSpace will scale in or scale out nodes in your cluster. See auto_scaling_configuration.
     */
    autoScalingConfiguration?: pulumi.Input<inputs.finspace.KxClusterAutoScalingConfiguration>;
    /**
     * The availability zone identifiers for the requested regions. Required when `azMode` is set to SINGLE.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The number of availability zones you want to assign per cluster. This can be one of the following:
     * * SINGLE - Assigns one availability zone per cluster.
     * * MULTI - Assigns all the availability zones per cluster.
     */
    azMode?: pulumi.Input<string>;
    /**
     * Configurations for a read only cache storage associated with a cluster. This cache will be stored as an FSx Lustre that reads from the S3 store. See cache_storage_configuration.
     */
    cacheStorageConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxClusterCacheStorageConfiguration>[]>;
    /**
     * Structure for the metadata of a cluster. Includes information like the CPUs needed, memory of instances, and number of instances. See capacity_configuration.
     */
    capacityConfiguration?: pulumi.Input<inputs.finspace.KxClusterCapacityConfiguration>;
    /**
     * Details of the custom code that you want to use inside a cluster when analyzing data. Consists of the S3 source bucket, location, object version, and the relative path from where the custom code is loaded into the cluster. See code.
     */
    code?: pulumi.Input<inputs.finspace.KxClusterCode>;
    /**
     * List of key-value pairs to make available inside the cluster.
     */
    commandLineArguments?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Timestamp at which the cluster is created in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    createdTimestamp?: pulumi.Input<string>;
    /**
     * KX database that will be available for querying. Defined below.
     */
    databases?: pulumi.Input<pulumi.Input<inputs.finspace.KxClusterDatabase>[]>;
    /**
     * Description of the cluster.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * An IAM role that defines a set of permissions associated with a cluster. These permissions are assumed when a cluster attempts to access another cluster.
     */
    executionRole?: pulumi.Input<string>;
    /**
     * Path to Q program that will be run at launch of a cluster. This is a relative path within .zip file that contains the custom code, which will be loaded on the cluster. It must include the file name itself. For example, somedir/init.q.
     */
    initializationScript?: pulumi.Input<string>;
    /**
     * Last timestamp at which the cluster was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    lastModifiedTimestamp?: pulumi.Input<string>;
    /**
     * Unique name for the cluster that you want to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Version of FinSpace Managed kdb to run.
     */
    releaseLabel?: pulumi.Input<string>;
    /**
     * Size and type of the temporary storage that is used to hold data during the savedown process. This parameter is required when you choose `type` as RDB. All the data written to this storage space is lost when the cluster node is restarted. See savedown_storage_configuration.
     */
    savedownStorageConfiguration?: pulumi.Input<inputs.finspace.KxClusterSavedownStorageConfiguration>;
    /**
     * The structure that stores the configuration details of a scaling group.
     */
    scalingGroupConfiguration?: pulumi.Input<inputs.finspace.KxClusterScalingGroupConfiguration>;
    status?: pulumi.Input<string>;
    statusReason?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A configuration to store Tickerplant logs. It consists of a list of volumes that will be mounted to your cluster. For the cluster type Tickerplant , the location of the TP volume on the cluster will be available by using the global variable .aws.tp_log_path.
     */
    tickerplantLogConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxClusterTickerplantLogConfiguration>[]>;
    /**
     * Type of KDB database. The following types are available:
     * * HDB - Historical Database. The data is only accessible with read-only permissions from one of the FinSpace managed KX databases mounted to the cluster.
     * * RDB - Realtime Database. This type of database captures all the data from a ticker plant and stores it in memory until the end of day, after which it writes all of its data to a disk and reloads the HDB. This cluster type requires local storage for temporary storage of data during the savedown process. If you specify this field in your request, you must provide the `savedownStorageConfiguration` parameter.
     * * GATEWAY - A gateway cluster allows you to access data across processes in kdb systems. It allows you to create your own routing logic using the initialization scripts and custom code. This type of cluster does not require a  writable local storage.
     * * GP - A general purpose cluster allows you to quickly iterate on code during development by granting greater access to system commands and enabling a fast reload of custom code. This cluster type can optionally mount databases including cache and savedown storage. For this cluster type, the node count is fixed at 1. It does not support autoscaling and supports only `SINGLE` AZ mode.
     * * Tickerplant - A tickerplant cluster allows you to subscribe to feed handlers based on IAM permissions. It can publish to RDBs, other Tickerplants, and real-time subscribers (RTS). Tickerplants can persist messages to log, which is readable by any RDB environment. It supports only single-node that is only one kdb process.
     */
    type?: pulumi.Input<string>;
    /**
     * Configuration details about the network where the Privatelink endpoint of the cluster resides. See vpc_configuration.
     *
     * The following arguments are optional:
     */
    vpcConfiguration?: pulumi.Input<inputs.finspace.KxClusterVpcConfiguration>;
}
/**
 * The set of arguments for constructing a KxCluster resource.
 */
export interface KxClusterArgs {
    /**
     * Configuration based on which FinSpace will scale in or scale out nodes in your cluster. See auto_scaling_configuration.
     */
    autoScalingConfiguration?: pulumi.Input<inputs.finspace.KxClusterAutoScalingConfiguration>;
    /**
     * The availability zone identifiers for the requested regions. Required when `azMode` is set to SINGLE.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The number of availability zones you want to assign per cluster. This can be one of the following:
     * * SINGLE - Assigns one availability zone per cluster.
     * * MULTI - Assigns all the availability zones per cluster.
     */
    azMode: pulumi.Input<string>;
    /**
     * Configurations for a read only cache storage associated with a cluster. This cache will be stored as an FSx Lustre that reads from the S3 store. See cache_storage_configuration.
     */
    cacheStorageConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxClusterCacheStorageConfiguration>[]>;
    /**
     * Structure for the metadata of a cluster. Includes information like the CPUs needed, memory of instances, and number of instances. See capacity_configuration.
     */
    capacityConfiguration?: pulumi.Input<inputs.finspace.KxClusterCapacityConfiguration>;
    /**
     * Details of the custom code that you want to use inside a cluster when analyzing data. Consists of the S3 source bucket, location, object version, and the relative path from where the custom code is loaded into the cluster. See code.
     */
    code?: pulumi.Input<inputs.finspace.KxClusterCode>;
    /**
     * List of key-value pairs to make available inside the cluster.
     */
    commandLineArguments?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * KX database that will be available for querying. Defined below.
     */
    databases?: pulumi.Input<pulumi.Input<inputs.finspace.KxClusterDatabase>[]>;
    /**
     * Description of the cluster.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId: pulumi.Input<string>;
    /**
     * An IAM role that defines a set of permissions associated with a cluster. These permissions are assumed when a cluster attempts to access another cluster.
     */
    executionRole?: pulumi.Input<string>;
    /**
     * Path to Q program that will be run at launch of a cluster. This is a relative path within .zip file that contains the custom code, which will be loaded on the cluster. It must include the file name itself. For example, somedir/init.q.
     */
    initializationScript?: pulumi.Input<string>;
    /**
     * Unique name for the cluster that you want to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Version of FinSpace Managed kdb to run.
     */
    releaseLabel: pulumi.Input<string>;
    /**
     * Size and type of the temporary storage that is used to hold data during the savedown process. This parameter is required when you choose `type` as RDB. All the data written to this storage space is lost when the cluster node is restarted. See savedown_storage_configuration.
     */
    savedownStorageConfiguration?: pulumi.Input<inputs.finspace.KxClusterSavedownStorageConfiguration>;
    /**
     * The structure that stores the configuration details of a scaling group.
     */
    scalingGroupConfiguration?: pulumi.Input<inputs.finspace.KxClusterScalingGroupConfiguration>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A configuration to store Tickerplant logs. It consists of a list of volumes that will be mounted to your cluster. For the cluster type Tickerplant , the location of the TP volume on the cluster will be available by using the global variable .aws.tp_log_path.
     */
    tickerplantLogConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxClusterTickerplantLogConfiguration>[]>;
    /**
     * Type of KDB database. The following types are available:
     * * HDB - Historical Database. The data is only accessible with read-only permissions from one of the FinSpace managed KX databases mounted to the cluster.
     * * RDB - Realtime Database. This type of database captures all the data from a ticker plant and stores it in memory until the end of day, after which it writes all of its data to a disk and reloads the HDB. This cluster type requires local storage for temporary storage of data during the savedown process. If you specify this field in your request, you must provide the `savedownStorageConfiguration` parameter.
     * * GATEWAY - A gateway cluster allows you to access data across processes in kdb systems. It allows you to create your own routing logic using the initialization scripts and custom code. This type of cluster does not require a  writable local storage.
     * * GP - A general purpose cluster allows you to quickly iterate on code during development by granting greater access to system commands and enabling a fast reload of custom code. This cluster type can optionally mount databases including cache and savedown storage. For this cluster type, the node count is fixed at 1. It does not support autoscaling and supports only `SINGLE` AZ mode.
     * * Tickerplant - A tickerplant cluster allows you to subscribe to feed handlers based on IAM permissions. It can publish to RDBs, other Tickerplants, and real-time subscribers (RTS). Tickerplants can persist messages to log, which is readable by any RDB environment. It supports only single-node that is only one kdb process.
     */
    type: pulumi.Input<string>;
    /**
     * Configuration details about the network where the Privatelink endpoint of the cluster resides. See vpc_configuration.
     *
     * The following arguments are optional:
     */
    vpcConfiguration: pulumi.Input<inputs.finspace.KxClusterVpcConfiguration>;
}
