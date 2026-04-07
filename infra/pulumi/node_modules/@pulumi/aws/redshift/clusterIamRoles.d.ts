import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Redshift Cluster IAM Roles resource.
 *
 * > **NOTE:** A Redshift cluster's default IAM role can be managed both by this resource's `defaultIamRoleArn` argument and the `aws.redshift.Cluster` resource's `defaultIamRoleArn` argument. Do not configure different values for both arguments. Doing so will cause a conflict of default IAM roles.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.ClusterIamRoles("example", {
 *     clusterIdentifier: exampleAwsRedshiftCluster.clusterIdentifier,
 *     iamRoleArns: [exampleAwsIamRole.arn],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Cluster IAM Roless using the `cluster_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/clusterIamRoles:ClusterIamRoles examplegroup1 example
 * ```
 */
export declare class ClusterIamRoles extends pulumi.CustomResource {
    /**
     * Get an existing ClusterIamRoles resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterIamRolesState, opts?: pulumi.CustomResourceOptions): ClusterIamRoles;
    /**
     * Returns true if the given object is an instance of ClusterIamRoles.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterIamRoles;
    /**
     * The name of the Redshift Cluster IAM Roles.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM role that was set as default for the cluster when the cluster was created.
     */
    readonly defaultIamRoleArn: pulumi.Output<string>;
    /**
     * A list of IAM Role ARNs to associate with the cluster. A Maximum of 10 can be associated to the cluster at any time.
     */
    readonly iamRoleArns: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ClusterIamRoles resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterIamRolesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterIamRoles resources.
 */
export interface ClusterIamRolesState {
    /**
     * The name of the Redshift Cluster IAM Roles.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM role that was set as default for the cluster when the cluster was created.
     */
    defaultIamRoleArn?: pulumi.Input<string>;
    /**
     * A list of IAM Role ARNs to associate with the cluster. A Maximum of 10 can be associated to the cluster at any time.
     */
    iamRoleArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ClusterIamRoles resource.
 */
export interface ClusterIamRolesArgs {
    /**
     * The name of the Redshift Cluster IAM Roles.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM role that was set as default for the cluster when the cluster was created.
     */
    defaultIamRoleArn?: pulumi.Input<string>;
    /**
     * A list of IAM Role ARNs to associate with the cluster. A Maximum of 10 can be associated to the cluster at any time.
     */
    iamRoleArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
