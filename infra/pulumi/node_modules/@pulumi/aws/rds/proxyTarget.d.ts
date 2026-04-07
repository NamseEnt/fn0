import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an RDS DB proxy target resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rds.Proxy("example", {
 *     name: "example",
 *     debugLogging: false,
 *     engineFamily: "MYSQL",
 *     idleClientTimeout: 1800,
 *     requireTls: true,
 *     roleArn: exampleAwsIamRole.arn,
 *     vpcSecurityGroupIds: [exampleAwsSecurityGroup.id],
 *     vpcSubnetIds: [exampleAwsSubnet.id],
 *     auths: [{
 *         authScheme: "SECRETS",
 *         description: "example",
 *         iamAuth: "DISABLED",
 *         secretArn: exampleAwsSecretsmanagerSecret.arn,
 *     }],
 *     tags: {
 *         Name: "example",
 *         Key: "value",
 *     },
 * });
 * const exampleProxyDefaultTargetGroup = new aws.rds.ProxyDefaultTargetGroup("example", {
 *     dbProxyName: example.name,
 *     connectionPoolConfig: {
 *         connectionBorrowTimeout: 120,
 *         initQuery: "SET x=1, y=2",
 *         maxConnectionsPercent: 100,
 *         maxIdleConnectionsPercent: 50,
 *         sessionPinningFilters: ["EXCLUDE_VARIABLE_SETS"],
 *     },
 * });
 * const exampleProxyTarget = new aws.rds.ProxyTarget("example", {
 *     dbInstanceIdentifier: exampleAwsDbInstance.identifier,
 *     dbProxyName: example.name,
 *     targetGroupName: exampleProxyDefaultTargetGroup.name,
 * });
 * ```
 *
 * ## Import
 *
 * Provisioned Clusters:
 *
 * __Using `pulumi import` to import__ RDS DB Proxy Targets using the `db_proxy_name`, `target_group_name`, target type (such as `RDS_INSTANCE` or `TRACKED_CLUSTER`), and resource identifier separated by forward slashes (`/`). For example:
 *
 * Instances:
 *
 * ```sh
 * $ pulumi import aws:rds/proxyTarget:ProxyTarget example example-proxy/default/RDS_INSTANCE/example-instance
 * ```
 * Provisioned Clusters:
 *
 * ```sh
 * $ pulumi import aws:rds/proxyTarget:ProxyTarget example example-proxy/default/TRACKED_CLUSTER/example-cluster
 * ```
 */
export declare class ProxyTarget extends pulumi.CustomResource {
    /**
     * Get an existing ProxyTarget resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProxyTargetState, opts?: pulumi.CustomResourceOptions): ProxyTarget;
    /**
     * Returns true if the given object is an instance of ProxyTarget.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProxyTarget;
    /**
     * DB cluster identifier.
     *
     * **NOTE:** Either `dbInstanceIdentifier` or `dbClusterIdentifier` should be specified and both should not be specified together
     */
    readonly dbClusterIdentifier: pulumi.Output<string | undefined>;
    /**
     * DB instance identifier.
     */
    readonly dbInstanceIdentifier: pulumi.Output<string | undefined>;
    /**
     * The name of the DB proxy.
     */
    readonly dbProxyName: pulumi.Output<string>;
    /**
     * Hostname for the target RDS DB Instance. Only returned for `RDS_INSTANCE` type.
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * Port for the target RDS DB Instance or Aurora DB Cluster.
     */
    readonly port: pulumi.Output<number>;
    /**
     * Identifier representing the DB Instance or DB Cluster target.
     */
    readonly rdsResourceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) for the DB instance or DB cluster. Currently not returned by the RDS API.
     */
    readonly targetArn: pulumi.Output<string>;
    /**
     * The name of the target group.
     */
    readonly targetGroupName: pulumi.Output<string>;
    /**
     * DB Cluster identifier for the DB Instance target. Not returned unless manually importing an `RDS_INSTANCE` target that is part of a DB Cluster.
     */
    readonly trackedClusterId: pulumi.Output<string>;
    /**
     * Type of targetE.g., `RDS_INSTANCE` or `TRACKED_CLUSTER`
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ProxyTarget resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProxyTargetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProxyTarget resources.
 */
export interface ProxyTargetState {
    /**
     * DB cluster identifier.
     *
     * **NOTE:** Either `dbInstanceIdentifier` or `dbClusterIdentifier` should be specified and both should not be specified together
     */
    dbClusterIdentifier?: pulumi.Input<string>;
    /**
     * DB instance identifier.
     */
    dbInstanceIdentifier?: pulumi.Input<string>;
    /**
     * The name of the DB proxy.
     */
    dbProxyName?: pulumi.Input<string>;
    /**
     * Hostname for the target RDS DB Instance. Only returned for `RDS_INSTANCE` type.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Port for the target RDS DB Instance or Aurora DB Cluster.
     */
    port?: pulumi.Input<number>;
    /**
     * Identifier representing the DB Instance or DB Cluster target.
     */
    rdsResourceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) for the DB instance or DB cluster. Currently not returned by the RDS API.
     */
    targetArn?: pulumi.Input<string>;
    /**
     * The name of the target group.
     */
    targetGroupName?: pulumi.Input<string>;
    /**
     * DB Cluster identifier for the DB Instance target. Not returned unless manually importing an `RDS_INSTANCE` target that is part of a DB Cluster.
     */
    trackedClusterId?: pulumi.Input<string>;
    /**
     * Type of targetE.g., `RDS_INSTANCE` or `TRACKED_CLUSTER`
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ProxyTarget resource.
 */
export interface ProxyTargetArgs {
    /**
     * DB cluster identifier.
     *
     * **NOTE:** Either `dbInstanceIdentifier` or `dbClusterIdentifier` should be specified and both should not be specified together
     */
    dbClusterIdentifier?: pulumi.Input<string>;
    /**
     * DB instance identifier.
     */
    dbInstanceIdentifier?: pulumi.Input<string>;
    /**
     * The name of the DB proxy.
     */
    dbProxyName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the target group.
     */
    targetGroupName: pulumi.Input<string>;
}
