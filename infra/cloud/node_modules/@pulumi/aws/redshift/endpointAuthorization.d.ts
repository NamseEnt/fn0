import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a new Amazon Redshift endpoint authorization.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.EndpointAuthorization("example", {
 *     account: "01234567910",
 *     clusterIdentifier: exampleAwsRedshiftCluster.clusterIdentifier,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift endpoint authorization using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/endpointAuthorization:EndpointAuthorization example 01234567910:cluster-example-id
 * ```
 */
export declare class EndpointAuthorization extends pulumi.CustomResource {
    /**
     * Get an existing EndpointAuthorization resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointAuthorizationState, opts?: pulumi.CustomResourceOptions): EndpointAuthorization;
    /**
     * Returns true if the given object is an instance of EndpointAuthorization.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EndpointAuthorization;
    /**
     * The Amazon Web Services account ID to grant access to.
     */
    readonly account: pulumi.Output<string>;
    /**
     * Indicates whether all VPCs in the grantee account are allowed access to the cluster.
     */
    readonly allowedAllVpcs: pulumi.Output<boolean>;
    /**
     * The cluster identifier of the cluster to grant access to.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The number of Redshift-managed VPC endpoints created for the authorization.
     */
    readonly endpointCount: pulumi.Output<number>;
    /**
     * Indicates whether to force the revoke action. If true, the Redshift-managed VPC endpoints associated with the endpoint authorization are also deleted. Default value is `false`.
     */
    readonly forceDelete: pulumi.Output<boolean | undefined>;
    /**
     * The Amazon Web Services account ID of the grantee of the cluster.
     */
    readonly grantee: pulumi.Output<string>;
    /**
     * The Amazon Web Services account ID of the cluster owner.
     */
    readonly grantor: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The virtual private cloud (VPC) identifiers to grant access to. If none are specified all VPCs in shared account are allowed.
     */
    readonly vpcIds: pulumi.Output<string[] | undefined>;
    /**
     * Create a EndpointAuthorization resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointAuthorizationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EndpointAuthorization resources.
 */
export interface EndpointAuthorizationState {
    /**
     * The Amazon Web Services account ID to grant access to.
     */
    account?: pulumi.Input<string>;
    /**
     * Indicates whether all VPCs in the grantee account are allowed access to the cluster.
     */
    allowedAllVpcs?: pulumi.Input<boolean>;
    /**
     * The cluster identifier of the cluster to grant access to.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The number of Redshift-managed VPC endpoints created for the authorization.
     */
    endpointCount?: pulumi.Input<number>;
    /**
     * Indicates whether to force the revoke action. If true, the Redshift-managed VPC endpoints associated with the endpoint authorization are also deleted. Default value is `false`.
     */
    forceDelete?: pulumi.Input<boolean>;
    /**
     * The Amazon Web Services account ID of the grantee of the cluster.
     */
    grantee?: pulumi.Input<string>;
    /**
     * The Amazon Web Services account ID of the cluster owner.
     */
    grantor?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The virtual private cloud (VPC) identifiers to grant access to. If none are specified all VPCs in shared account are allowed.
     */
    vpcIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a EndpointAuthorization resource.
 */
export interface EndpointAuthorizationArgs {
    /**
     * The Amazon Web Services account ID to grant access to.
     */
    account: pulumi.Input<string>;
    /**
     * The cluster identifier of the cluster to grant access to.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * Indicates whether to force the revoke action. If true, the Redshift-managed VPC endpoints associated with the endpoint authorization are also deleted. Default value is `false`.
     */
    forceDelete?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The virtual private cloud (VPC) identifiers to grant access to. If none are specified all VPCs in shared account are allowed.
     */
    vpcIds?: pulumi.Input<pulumi.Input<string>[]>;
}
