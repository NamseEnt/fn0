import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an EKS Identity Provider Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.eks.IdentityProviderConfig("example", {
 *     clusterName: exampleAwsEksCluster.name,
 *     oidc: {
 *         clientId: "your client_id",
 *         identityProviderConfigName: "example",
 *         issuerUrl: "your issuer_url",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EKS Identity Provider Configurations using the `cluster_name` and `identity_provider_config_name` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:eks/identityProviderConfig:IdentityProviderConfig my_identity_provider_config my_cluster:my_identity_provider_config
 * ```
 */
export declare class IdentityProviderConfig extends pulumi.CustomResource {
    /**
     * Get an existing IdentityProviderConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IdentityProviderConfigState, opts?: pulumi.CustomResourceOptions): IdentityProviderConfig;
    /**
     * Returns true if the given object is an instance of IdentityProviderConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IdentityProviderConfig;
    /**
     * Amazon Resource Name (ARN) of the EKS Identity Provider Configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of the EKS Cluster.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * Nested attribute containing [OpenID Connect](https://openid.net/connect/) identity provider information for the cluster. Detailed below.
     */
    readonly oidc: pulumi.Output<outputs.eks.IdentityProviderConfigOidc>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the EKS Identity Provider Configuration.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a IdentityProviderConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IdentityProviderConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IdentityProviderConfig resources.
 */
export interface IdentityProviderConfigState {
    /**
     * Amazon Resource Name (ARN) of the EKS Identity Provider Configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of the EKS Cluster.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * Nested attribute containing [OpenID Connect](https://openid.net/connect/) identity provider information for the cluster. Detailed below.
     */
    oidc?: pulumi.Input<inputs.eks.IdentityProviderConfigOidc>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the EKS Identity Provider Configuration.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a IdentityProviderConfig resource.
 */
export interface IdentityProviderConfigArgs {
    /**
     * Name of the EKS Cluster.
     */
    clusterName: pulumi.Input<string>;
    /**
     * Nested attribute containing [OpenID Connect](https://openid.net/connect/) identity provider information for the cluster. Detailed below.
     */
    oidc: pulumi.Input<inputs.eks.IdentityProviderConfigOidc>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
