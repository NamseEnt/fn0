import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates an Amazon CloudHSM v2 cluster.
 *
 * For information about CloudHSM v2, see the
 * [AWS CloudHSM User Guide](https://docs.aws.amazon.com/cloudhsm/latest/userguide/introduction.html) and the [Amazon
 * CloudHSM API Reference][2].
 *
 * > **NOTE:** A CloudHSM Cluster can take several minutes to set up.
 * Practically no single attribute can be updated, except for `tags`.
 * If you need to delete a cluster, you have to remove its HSM modules first.
 * To initialize cluster, you have to add an HSM instance to the cluster, then sign CSR and upload it.
 *
 * ## Example Usage
 *
 * The following example below creates a CloudHSM cluster.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.getAvailabilityZones({});
 * const cloudhsmV2Vpc = new aws.ec2.Vpc("cloudhsm_v2_vpc", {
 *     cidrBlock: "10.0.0.0/16",
 *     tags: {
 *         Name: "example-aws_cloudhsm_v2_cluster",
 *     },
 * });
 * const cloudhsmV2Subnets: aws.ec2.Subnet[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     cloudhsmV2Subnets.push(new aws.ec2.Subnet(`cloudhsm_v2_subnets-${range.value}`, {
 *         vpcId: cloudhsmV2Vpc.id,
 *         cidrBlock: subnets[range.value],
 *         mapPublicIpOnLaunch: false,
 *         availabilityZone: available.then(available => available.names[range.value]),
 *         tags: {
 *             Name: "example-aws_cloudhsm_v2_cluster",
 *         },
 *     }));
 * }
 * const cloudhsmV2Cluster = new aws.cloudhsmv2.Cluster("cloudhsm_v2_cluster", {
 *     hsmType: "hsm1.medium",
 *     subnetIds: cloudhsmV2Subnets.map(__item => __item.id),
 *     tags: {
 *         Name: "example-aws_cloudhsm_v2_cluster",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudHSM v2 Clusters using the cluster `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudhsmv2/cluster:Cluster test_cluster cluster-aeb282a201
 * ```
 */
export declare class Cluster extends pulumi.CustomResource {
    /**
     * Get an existing Cluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterState, opts?: pulumi.CustomResourceOptions): Cluster;
    /**
     * Returns true if the given object is an instance of Cluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Cluster;
    /**
     * The list of cluster certificates.
     */
    readonly clusterCertificates: pulumi.Output<outputs.cloudhsmv2.ClusterClusterCertificate[]>;
    /**
     * The id of the CloudHSM cluster.
     */
    readonly clusterId: pulumi.Output<string>;
    /**
     * The state of the CloudHSM cluster.
     */
    readonly clusterState: pulumi.Output<string>;
    /**
     * The type of HSM module in the cluster. Currently, `hsm1.medium` and `hsm2m.medium` are supported.
     */
    readonly hsmType: pulumi.Output<string>;
    /**
     * The mode to use in the cluster. The allowed values are `FIPS` and `NON_FIPS`. This field is required if `hsmType` is `hsm2m.medium`.
     */
    readonly mode: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the security group associated with the CloudHSM cluster.
     */
    readonly securityGroupId: pulumi.Output<string>;
    /**
     * ID of Cloud HSM v2 cluster backup to be restored.
     */
    readonly sourceBackupIdentifier: pulumi.Output<string | undefined>;
    /**
     * The IDs of subnets in which cluster will operate.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The id of the VPC that the CloudHSM cluster resides in.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * The list of cluster certificates.
     */
    clusterCertificates?: pulumi.Input<pulumi.Input<inputs.cloudhsmv2.ClusterClusterCertificate>[]>;
    /**
     * The id of the CloudHSM cluster.
     */
    clusterId?: pulumi.Input<string>;
    /**
     * The state of the CloudHSM cluster.
     */
    clusterState?: pulumi.Input<string>;
    /**
     * The type of HSM module in the cluster. Currently, `hsm1.medium` and `hsm2m.medium` are supported.
     */
    hsmType?: pulumi.Input<string>;
    /**
     * The mode to use in the cluster. The allowed values are `FIPS` and `NON_FIPS`. This field is required if `hsmType` is `hsm2m.medium`.
     */
    mode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the security group associated with the CloudHSM cluster.
     */
    securityGroupId?: pulumi.Input<string>;
    /**
     * ID of Cloud HSM v2 cluster backup to be restored.
     */
    sourceBackupIdentifier?: pulumi.Input<string>;
    /**
     * The IDs of subnets in which cluster will operate.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The id of the VPC that the CloudHSM cluster resides in.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * The type of HSM module in the cluster. Currently, `hsm1.medium` and `hsm2m.medium` are supported.
     */
    hsmType: pulumi.Input<string>;
    /**
     * The mode to use in the cluster. The allowed values are `FIPS` and `NON_FIPS`. This field is required if `hsmType` is `hsm2m.medium`.
     */
    mode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of Cloud HSM v2 cluster backup to be restored.
     */
    sourceBackupIdentifier?: pulumi.Input<string>;
    /**
     * The IDs of subnets in which cluster will operate.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
