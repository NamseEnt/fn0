import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing exadata infrastructure resource in AWS for Oracle Database@AWS.
 *
 * ## Example Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import Exadata Infrastructure using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:odb/cloudExadataInfrastructure:CloudExadataInfrastructure example example
 * ```
 */
export declare class CloudExadataInfrastructure extends pulumi.CustomResource {
    /**
     * Get an existing CloudExadataInfrastructure resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CloudExadataInfrastructureState, opts?: pulumi.CustomResourceOptions): CloudExadataInfrastructure;
    /**
     * Returns true if the given object is an instance of CloudExadataInfrastructure.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CloudExadataInfrastructure;
    /**
     * The number of storage servers requested for the Exadata infrastructure.
     */
    readonly activatedStorageCount: pulumi.Output<number>;
    /**
     * The number of storage servers requested for the Exadata infrastructure.
     */
    readonly additionalStorageCount: pulumi.Output<number>;
    /**
     * Amazon Resource Name (ARN) of the Exadata infrastructure.
     */
    readonly arn: pulumi.Output<string>;
    readonly availabilityZone: pulumi.Output<string>;
    readonly availabilityZoneId: pulumi.Output<string>;
    /**
     * The amount of available storage, in gigabytes (GB), for the Exadata infrastructure.
     */
    readonly availableStorageSizeInGbs: pulumi.Output<number>;
    /**
     * The number of compute instances that the Exadata infrastructure is located
     */
    readonly computeCount: pulumi.Output<number>;
    /**
     * The OCI model compute model used when you create or clone an instance: ECPU or OCPU.
     */
    readonly computeModel: pulumi.Output<string>;
    /**
     * The total number of CPU cores that are allocated to the Exadata infrastructure.
     */
    readonly cpuCount: pulumi.Output<number>;
    /**
     * The time when the Exadata infrastructure was created.
     */
    readonly createdAt: pulumi.Output<string>;
    readonly customerContactsToSendToOcis: pulumi.Output<outputs.odb.CloudExadataInfrastructureCustomerContactsToSendToOci[] | undefined>;
    /**
     * The size of the Exadata infrastructure's data disk group, in terabytes (TB).
     */
    readonly dataStorageSizeInTbs: pulumi.Output<number>;
    /**
     * The database server model type of the Exadata infrastructure. For the list of valid model names, use the ListDbSystemShapes operation
     */
    readonly databaseServerType: pulumi.Output<string | undefined>;
    /**
     * The size of the Exadata infrastructure's local node storage, in gigabytes (GB).
     */
    readonly dbNodeStorageSizeInGbs: pulumi.Output<number>;
    /**
     * The software version of the database servers (dom0) in the Exadata infrastructure.
     */
    readonly dbServerVersion: pulumi.Output<string>;
    readonly displayName: pulumi.Output<string>;
    /**
     * The Oracle Cloud Identifier (OCID) of the last maintenance run for the Exadata infrastructure.
     */
    readonly lastMaintenanceRunId: pulumi.Output<string>;
    /**
     * The scheduling details for the maintenance window. Patching and system updates take place during the maintenance window
     */
    readonly maintenanceWindow: pulumi.Output<outputs.odb.CloudExadataInfrastructureMaintenanceWindow | undefined>;
    /**
     * The total number of CPU cores available on the Exadata infrastructure.
     */
    readonly maxCpuCount: pulumi.Output<number>;
    /**
     * The total amount of data disk group storage, in terabytes (TB), that's available on the Exadata infrastructure.
     */
    readonly maxDataStorageInTbs: pulumi.Output<number>;
    /**
     * The total amount of local node storage, in gigabytes (GB), that's available on the Exadata infrastructure.
     */
    readonly maxDbNodeStorageSizeInGbs: pulumi.Output<number>;
    /**
     * The total amount of memory in gigabytes (GB) available on the Exadata infrastructure.
     */
    readonly maxMemoryInGbs: pulumi.Output<number>;
    /**
     * The amount of memory, in gigabytes (GB), that's allocated on the Exadata infrastructure
     */
    readonly memorySizeInGbs: pulumi.Output<number>;
    /**
     * The monthly software version of the database servers in the Exadata infrastructure.
     */
    readonly monthlyDbServerVersion: pulumi.Output<string>;
    /**
     * The monthly software version of the storage servers installed on the Exadata infrastructure.
     */
    readonly monthlyStorageServerVersion: pulumi.Output<string>;
    /**
     * The OCID of the next maintenance run for the Exadata infrastructure.
     */
    readonly nextMaintenanceRunId: pulumi.Output<string>;
    /**
     * The name of the OCI resource anchor for the Exadata infrastructure.
     */
    readonly ociResourceAnchorName: pulumi.Output<string>;
    /**
     * The HTTPS link to the Exadata infrastructure in OCI
     */
    readonly ociUrl: pulumi.Output<string>;
    /**
     * The OCID of the Exadata infrastructure.
     */
    readonly ocid: pulumi.Output<string>;
    /**
     * The amount of progress made on the current operation on the Exadata infrastructure, expressed as a percentage.
     */
    readonly percentProgress: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly shape: pulumi.Output<string>;
    /**
     * The current status of the Exadata infrastructure.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Additional information about the status of the Exadata infrastructure.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * TThe number of storage servers that are activated for the Exadata infrastructure
     */
    readonly storageCount: pulumi.Output<number>;
    /**
     * The storage server model type of the Exadata infrastructure. For the list of valid model names, use the ListDbSystemShapes operation
     */
    readonly storageServerType: pulumi.Output<string | undefined>;
    /**
     * The software version of the storage servers on the Exadata infrastructure.
     */
    readonly storageServerVersion: pulumi.Output<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.odb.CloudExadataInfrastructureTimeouts | undefined>;
    /**
     * The total amount of storage, in gigabytes (GB), on the Exadata infrastructure.
     */
    readonly totalStorageSizeInGbs: pulumi.Output<number>;
    /**
     * Create a CloudExadataInfrastructure resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CloudExadataInfrastructureArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CloudExadataInfrastructure resources.
 */
export interface CloudExadataInfrastructureState {
    /**
     * The number of storage servers requested for the Exadata infrastructure.
     */
    activatedStorageCount?: pulumi.Input<number>;
    /**
     * The number of storage servers requested for the Exadata infrastructure.
     */
    additionalStorageCount?: pulumi.Input<number>;
    /**
     * Amazon Resource Name (ARN) of the Exadata infrastructure.
     */
    arn?: pulumi.Input<string>;
    availabilityZone?: pulumi.Input<string>;
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The amount of available storage, in gigabytes (GB), for the Exadata infrastructure.
     */
    availableStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The number of compute instances that the Exadata infrastructure is located
     */
    computeCount?: pulumi.Input<number>;
    /**
     * The OCI model compute model used when you create or clone an instance: ECPU or OCPU.
     */
    computeModel?: pulumi.Input<string>;
    /**
     * The total number of CPU cores that are allocated to the Exadata infrastructure.
     */
    cpuCount?: pulumi.Input<number>;
    /**
     * The time when the Exadata infrastructure was created.
     */
    createdAt?: pulumi.Input<string>;
    customerContactsToSendToOcis?: pulumi.Input<pulumi.Input<inputs.odb.CloudExadataInfrastructureCustomerContactsToSendToOci>[]>;
    /**
     * The size of the Exadata infrastructure's data disk group, in terabytes (TB).
     */
    dataStorageSizeInTbs?: pulumi.Input<number>;
    /**
     * The database server model type of the Exadata infrastructure. For the list of valid model names, use the ListDbSystemShapes operation
     */
    databaseServerType?: pulumi.Input<string>;
    /**
     * The size of the Exadata infrastructure's local node storage, in gigabytes (GB).
     */
    dbNodeStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The software version of the database servers (dom0) in the Exadata infrastructure.
     */
    dbServerVersion?: pulumi.Input<string>;
    displayName?: pulumi.Input<string>;
    /**
     * The Oracle Cloud Identifier (OCID) of the last maintenance run for the Exadata infrastructure.
     */
    lastMaintenanceRunId?: pulumi.Input<string>;
    /**
     * The scheduling details for the maintenance window. Patching and system updates take place during the maintenance window
     */
    maintenanceWindow?: pulumi.Input<inputs.odb.CloudExadataInfrastructureMaintenanceWindow>;
    /**
     * The total number of CPU cores available on the Exadata infrastructure.
     */
    maxCpuCount?: pulumi.Input<number>;
    /**
     * The total amount of data disk group storage, in terabytes (TB), that's available on the Exadata infrastructure.
     */
    maxDataStorageInTbs?: pulumi.Input<number>;
    /**
     * The total amount of local node storage, in gigabytes (GB), that's available on the Exadata infrastructure.
     */
    maxDbNodeStorageSizeInGbs?: pulumi.Input<number>;
    /**
     * The total amount of memory in gigabytes (GB) available on the Exadata infrastructure.
     */
    maxMemoryInGbs?: pulumi.Input<number>;
    /**
     * The amount of memory, in gigabytes (GB), that's allocated on the Exadata infrastructure
     */
    memorySizeInGbs?: pulumi.Input<number>;
    /**
     * The monthly software version of the database servers in the Exadata infrastructure.
     */
    monthlyDbServerVersion?: pulumi.Input<string>;
    /**
     * The monthly software version of the storage servers installed on the Exadata infrastructure.
     */
    monthlyStorageServerVersion?: pulumi.Input<string>;
    /**
     * The OCID of the next maintenance run for the Exadata infrastructure.
     */
    nextMaintenanceRunId?: pulumi.Input<string>;
    /**
     * The name of the OCI resource anchor for the Exadata infrastructure.
     */
    ociResourceAnchorName?: pulumi.Input<string>;
    /**
     * The HTTPS link to the Exadata infrastructure in OCI
     */
    ociUrl?: pulumi.Input<string>;
    /**
     * The OCID of the Exadata infrastructure.
     */
    ocid?: pulumi.Input<string>;
    /**
     * The amount of progress made on the current operation on the Exadata infrastructure, expressed as a percentage.
     */
    percentProgress?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    shape?: pulumi.Input<string>;
    /**
     * The current status of the Exadata infrastructure.
     */
    status?: pulumi.Input<string>;
    /**
     * Additional information about the status of the Exadata infrastructure.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * TThe number of storage servers that are activated for the Exadata infrastructure
     */
    storageCount?: pulumi.Input<number>;
    /**
     * The storage server model type of the Exadata infrastructure. For the list of valid model names, use the ListDbSystemShapes operation
     */
    storageServerType?: pulumi.Input<string>;
    /**
     * The software version of the storage servers on the Exadata infrastructure.
     */
    storageServerVersion?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.CloudExadataInfrastructureTimeouts>;
    /**
     * The total amount of storage, in gigabytes (GB), on the Exadata infrastructure.
     */
    totalStorageSizeInGbs?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a CloudExadataInfrastructure resource.
 */
export interface CloudExadataInfrastructureArgs {
    availabilityZone?: pulumi.Input<string>;
    availabilityZoneId: pulumi.Input<string>;
    /**
     * The number of compute instances that the Exadata infrastructure is located
     */
    computeCount?: pulumi.Input<number>;
    customerContactsToSendToOcis?: pulumi.Input<pulumi.Input<inputs.odb.CloudExadataInfrastructureCustomerContactsToSendToOci>[]>;
    /**
     * The database server model type of the Exadata infrastructure. For the list of valid model names, use the ListDbSystemShapes operation
     */
    databaseServerType?: pulumi.Input<string>;
    displayName: pulumi.Input<string>;
    /**
     * The scheduling details for the maintenance window. Patching and system updates take place during the maintenance window
     */
    maintenanceWindow?: pulumi.Input<inputs.odb.CloudExadataInfrastructureMaintenanceWindow>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    shape: pulumi.Input<string>;
    /**
     * TThe number of storage servers that are activated for the Exadata infrastructure
     */
    storageCount?: pulumi.Input<number>;
    /**
     * The storage server model type of the Exadata infrastructure. For the list of valid model names, use the ListDbSystemShapes operation
     */
    storageServerType?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.CloudExadataInfrastructureTimeouts>;
}
