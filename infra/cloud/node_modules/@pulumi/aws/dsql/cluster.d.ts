import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon Aurora DSQL Cluster.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dsql.Cluster("example", {
 *     deletionProtectionEnabled: true,
 *     tags: {
 *         Name: "TestCluster",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DSQL Cluster using the `identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:dsql/cluster:Cluster example abcde1f234ghijklmnop5qr6st
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
     * ARN of the Cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether deletion protection is enabled in this cluster.
     * Default value is `false`.
     */
    readonly deletionProtectionEnabled: pulumi.Output<boolean>;
    /**
     * Encryption configuration details for the DSQL Cluster.
     */
    readonly encryptionDetails: pulumi.Output<outputs.dsql.ClusterEncryptionDetail[]>;
    /**
     * Destroys cluster even if `deletionProtectionEnabled` is set to `true`.
     * Default value is `false`.
     */
    readonly forceDestroy: pulumi.Output<boolean>;
    /**
     * Cluster Identifier.
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * The ARN of the AWS KMS key that encrypts data in the DSQL Cluster, or `"AWS_OWNED_KMS_KEY"`.
     */
    readonly kmsEncryptionKey: pulumi.Output<string>;
    /**
     * Multi-region properties of the DSQL Cluster.
     */
    readonly multiRegionProperties: pulumi.Output<outputs.dsql.ClusterMultiRegionProperties | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of tags to be associated with the AWS DSQL Cluster resource.
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
    readonly timeouts: pulumi.Output<outputs.dsql.ClusterTimeouts | undefined>;
    /**
     * The DSQL Cluster's VPC endpoint service name.
     */
    readonly vpcEndpointServiceName: pulumi.Output<string>;
    /**
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * ARN of the Cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether deletion protection is enabled in this cluster.
     * Default value is `false`.
     */
    deletionProtectionEnabled?: pulumi.Input<boolean>;
    /**
     * Encryption configuration details for the DSQL Cluster.
     */
    encryptionDetails?: pulumi.Input<pulumi.Input<inputs.dsql.ClusterEncryptionDetail>[]>;
    /**
     * Destroys cluster even if `deletionProtectionEnabled` is set to `true`.
     * Default value is `false`.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Cluster Identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * The ARN of the AWS KMS key that encrypts data in the DSQL Cluster, or `"AWS_OWNED_KMS_KEY"`.
     */
    kmsEncryptionKey?: pulumi.Input<string>;
    /**
     * Multi-region properties of the DSQL Cluster.
     */
    multiRegionProperties?: pulumi.Input<inputs.dsql.ClusterMultiRegionProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of tags to be associated with the AWS DSQL Cluster resource.
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
    timeouts?: pulumi.Input<inputs.dsql.ClusterTimeouts>;
    /**
     * The DSQL Cluster's VPC endpoint service name.
     */
    vpcEndpointServiceName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * Whether deletion protection is enabled in this cluster.
     * Default value is `false`.
     */
    deletionProtectionEnabled?: pulumi.Input<boolean>;
    /**
     * Destroys cluster even if `deletionProtectionEnabled` is set to `true`.
     * Default value is `false`.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The ARN of the AWS KMS key that encrypts data in the DSQL Cluster, or `"AWS_OWNED_KMS_KEY"`.
     */
    kmsEncryptionKey?: pulumi.Input<string>;
    /**
     * Multi-region properties of the DSQL Cluster.
     */
    multiRegionProperties?: pulumi.Input<inputs.dsql.ClusterMultiRegionProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of tags to be associated with the AWS DSQL Cluster resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.dsql.ClusterTimeouts>;
}
