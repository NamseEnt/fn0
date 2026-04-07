import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a RDS DB Cluster association with an IAM Role. Example use cases:
 *
 * * [Creating an IAM Role to Allow Amazon Aurora to Access AWS Services](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/AuroraMySQL.Integrating.Authorizing.IAM.CreateRole.html)
 * * [Importing Amazon S3 Data into an RDS PostgreSQL DB Cluster](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_PostgreSQL.S3Import.html)
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rds.ClusterRoleAssociation("example", {
 *     dbClusterIdentifier: exampleAwsRdsCluster.id,
 *     featureName: "S3_INTEGRATION",
 *     roleArn: exampleAwsIamRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_rds_cluster_role_association` using the DB Cluster Identifier and IAM Role ARN separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:rds/clusterRoleAssociation:ClusterRoleAssociation example my-db-cluster,arn:aws:iam::123456789012:role/my-role
 * ```
 */
export declare class ClusterRoleAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ClusterRoleAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterRoleAssociationState, opts?: pulumi.CustomResourceOptions): ClusterRoleAssociation;
    /**
     * Returns true if the given object is an instance of ClusterRoleAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterRoleAssociation;
    /**
     * DB Cluster Identifier to associate with the IAM Role.
     */
    readonly dbClusterIdentifier: pulumi.Output<string>;
    /**
     * Name of the feature for association. This can be found in the AWS documentation relevant to the integration or a full list is available in the `SupportedFeatureNames` list returned by [AWS CLI rds describe-db-engine-versions](https://docs.aws.amazon.com/cli/latest/reference/rds/describe-db-engine-versions.html).
     */
    readonly featureName: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role to associate with the DB Cluster.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Create a ClusterRoleAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterRoleAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterRoleAssociation resources.
 */
export interface ClusterRoleAssociationState {
    /**
     * DB Cluster Identifier to associate with the IAM Role.
     */
    dbClusterIdentifier?: pulumi.Input<string>;
    /**
     * Name of the feature for association. This can be found in the AWS documentation relevant to the integration or a full list is available in the `SupportedFeatureNames` list returned by [AWS CLI rds describe-db-engine-versions](https://docs.aws.amazon.com/cli/latest/reference/rds/describe-db-engine-versions.html).
     */
    featureName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role to associate with the DB Cluster.
     */
    roleArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ClusterRoleAssociation resource.
 */
export interface ClusterRoleAssociationArgs {
    /**
     * DB Cluster Identifier to associate with the IAM Role.
     */
    dbClusterIdentifier: pulumi.Input<string>;
    /**
     * Name of the feature for association. This can be found in the AWS documentation relevant to the integration or a full list is available in the `SupportedFeatureNames` list returned by [AWS CLI rds describe-db-engine-versions](https://docs.aws.amazon.com/cli/latest/reference/rds/describe-db-engine-versions.html).
     */
    featureName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role to associate with the DB Cluster.
     */
    roleArn: pulumi.Input<string>;
}
