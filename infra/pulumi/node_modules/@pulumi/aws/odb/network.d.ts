import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing odb Network resource in AWS for Oracle Database@AWS.
 *
 * ## Example Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import Odb Network using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:odb/network:Network example example
 * ```
 */
export declare class Network extends pulumi.CustomResource {
    /**
     * Get an existing Network resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkState, opts?: pulumi.CustomResourceOptions): Network;
    /**
     * Returns true if the given object is an instance of Network.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Network;
    /**
     * Amazon Resource Name (ARN) of the odb network resource.
     */
    readonly arn: pulumi.Output<string>;
    readonly availabilityZone: pulumi.Output<string>;
    readonly availabilityZoneId: pulumi.Output<string>;
    readonly backupSubnetCidr: pulumi.Output<string>;
    readonly clientSubnetCidr: pulumi.Output<string>;
    /**
     * The date and time when the ODB network was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The name of the custom domain that the network is located. customDomainName and defaultDnsPrefix both can't be given.
     */
    readonly customDomainName: pulumi.Output<string | undefined>;
    readonly defaultDnsPrefix: pulumi.Output<string | undefined>;
    /**
     * If set to true deletes associated OCI resources. Default false.
     */
    readonly deleteAssociatedResources: pulumi.Output<boolean>;
    readonly displayName: pulumi.Output<string>;
    /**
     * The name of the OCI resource anchor for the Exadata infrastructure.
     */
    readonly managedServices: pulumi.Output<outputs.odb.NetworkManagedService[]>;
    /**
     * The number of storage servers requested for the Exadata infrastructure.
     */
    readonly ociDnsForwardingConfigs: pulumi.Output<outputs.odb.NetworkOciDnsForwardingConfig[]>;
    /**
     * The unique identifier of the OCI network anchor for the ODB network.
     */
    readonly ociNetworkAnchorId: pulumi.Output<string>;
    /**
     * The URL of the OCI network anchor for the ODB network.
     */
    readonly ociNetworkAnchorUrl: pulumi.Output<string>;
    /**
     * The name of the OCI resource anchor for the ODB network.
     */
    readonly ociResourceAnchorName: pulumi.Output<string>;
    /**
     * The unique identifier  Oracle Cloud ID (OCID) of the OCI VCN for the ODB network.
     */
    readonly ociVcnId: pulumi.Output<string>;
    /**
     * The URL of the OCI VCN for the ODB network.
     */
    readonly ociVcnUrl: pulumi.Output<string>;
    /**
     * The list of CIDR ranges from the peered VPC that are allowed access to the ODB network. Please refer odb network peering documentation.
     */
    readonly peeredCidrs: pulumi.Output<string[]>;
    /**
     * The amount of progress made on the current operation on the ODB network, expressed as a percentage.
     */
    readonly percentProgress: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the configuration for Amazon S3 access from the ODB network.
     */
    readonly s3Access: pulumi.Output<string>;
    /**
     * Specifies the endpoint policy for Amazon S3 access from the ODB network.
     */
    readonly s3PolicyDocument: pulumi.Output<string | undefined>;
    /**
     * The status of the network resource.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Additional information about the current status of the ODB network.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.odb.NetworkTimeouts | undefined>;
    /**
     * Specifies the configuration for Zero-ETL access from the ODB network.
     *
     * The following arguments are optional:
     */
    readonly zeroEtlAccess: pulumi.Output<string>;
    /**
     * Create a Network resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Network resources.
 */
export interface NetworkState {
    /**
     * Amazon Resource Name (ARN) of the odb network resource.
     */
    arn?: pulumi.Input<string>;
    availabilityZone?: pulumi.Input<string>;
    availabilityZoneId?: pulumi.Input<string>;
    backupSubnetCidr?: pulumi.Input<string>;
    clientSubnetCidr?: pulumi.Input<string>;
    /**
     * The date and time when the ODB network was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The name of the custom domain that the network is located. customDomainName and defaultDnsPrefix both can't be given.
     */
    customDomainName?: pulumi.Input<string>;
    defaultDnsPrefix?: pulumi.Input<string>;
    /**
     * If set to true deletes associated OCI resources. Default false.
     */
    deleteAssociatedResources?: pulumi.Input<boolean>;
    displayName?: pulumi.Input<string>;
    /**
     * The name of the OCI resource anchor for the Exadata infrastructure.
     */
    managedServices?: pulumi.Input<pulumi.Input<inputs.odb.NetworkManagedService>[]>;
    /**
     * The number of storage servers requested for the Exadata infrastructure.
     */
    ociDnsForwardingConfigs?: pulumi.Input<pulumi.Input<inputs.odb.NetworkOciDnsForwardingConfig>[]>;
    /**
     * The unique identifier of the OCI network anchor for the ODB network.
     */
    ociNetworkAnchorId?: pulumi.Input<string>;
    /**
     * The URL of the OCI network anchor for the ODB network.
     */
    ociNetworkAnchorUrl?: pulumi.Input<string>;
    /**
     * The name of the OCI resource anchor for the ODB network.
     */
    ociResourceAnchorName?: pulumi.Input<string>;
    /**
     * The unique identifier  Oracle Cloud ID (OCID) of the OCI VCN for the ODB network.
     */
    ociVcnId?: pulumi.Input<string>;
    /**
     * The URL of the OCI VCN for the ODB network.
     */
    ociVcnUrl?: pulumi.Input<string>;
    /**
     * The list of CIDR ranges from the peered VPC that are allowed access to the ODB network. Please refer odb network peering documentation.
     */
    peeredCidrs?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The amount of progress made on the current operation on the ODB network, expressed as a percentage.
     */
    percentProgress?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the configuration for Amazon S3 access from the ODB network.
     */
    s3Access?: pulumi.Input<string>;
    /**
     * Specifies the endpoint policy for Amazon S3 access from the ODB network.
     */
    s3PolicyDocument?: pulumi.Input<string>;
    /**
     * The status of the network resource.
     */
    status?: pulumi.Input<string>;
    /**
     * Additional information about the current status of the ODB network.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.NetworkTimeouts>;
    /**
     * Specifies the configuration for Zero-ETL access from the ODB network.
     *
     * The following arguments are optional:
     */
    zeroEtlAccess?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Network resource.
 */
export interface NetworkArgs {
    availabilityZone?: pulumi.Input<string>;
    availabilityZoneId: pulumi.Input<string>;
    backupSubnetCidr: pulumi.Input<string>;
    clientSubnetCidr: pulumi.Input<string>;
    /**
     * The name of the custom domain that the network is located. customDomainName and defaultDnsPrefix both can't be given.
     */
    customDomainName?: pulumi.Input<string>;
    defaultDnsPrefix?: pulumi.Input<string>;
    /**
     * If set to true deletes associated OCI resources. Default false.
     */
    deleteAssociatedResources?: pulumi.Input<boolean>;
    displayName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the configuration for Amazon S3 access from the ODB network.
     */
    s3Access: pulumi.Input<string>;
    /**
     * Specifies the endpoint policy for Amazon S3 access from the ODB network.
     */
    s3PolicyDocument?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exadata infrastructure. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.NetworkTimeouts>;
    /**
     * Specifies the configuration for Zero-ETL access from the ODB network.
     *
     * The following arguments are optional:
     */
    zeroEtlAccess: pulumi.Input<string>;
}
