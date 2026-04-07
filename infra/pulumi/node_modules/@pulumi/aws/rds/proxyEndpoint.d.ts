import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an RDS DB proxy endpoint resource. For additional information, see the [RDS User Guide](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/rds-proxy-endpoints.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rds.ProxyEndpoint("example", {
 *     dbProxyName: test.name,
 *     dbProxyEndpointName: "example",
 *     vpcSubnetIds: testAwsSubnet.map(__item => __item.id),
 *     targetRole: "READ_ONLY",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DB proxy endpoints using the `DB-PROXY-NAME/DB-PROXY-ENDPOINT-NAME`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/proxyEndpoint:ProxyEndpoint example example/example
 * ```
 */
export declare class ProxyEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing ProxyEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProxyEndpointState, opts?: pulumi.CustomResourceOptions): ProxyEndpoint;
    /**
     * Returns true if the given object is an instance of ProxyEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProxyEndpoint;
    /**
     * The Amazon Resource Name (ARN) for the proxy endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The identifier for the proxy endpoint. An identifier must begin with a letter and must contain only ASCII letters, digits, and hyphens; it can't end with a hyphen or contain two consecutive hyphens.
     */
    readonly dbProxyEndpointName: pulumi.Output<string>;
    /**
     * The name of the DB proxy associated with the DB proxy endpoint that you create.
     */
    readonly dbProxyName: pulumi.Output<string>;
    /**
     * The endpoint that you can use to connect to the proxy. You include the endpoint value in the connection string for a database client application.
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * Indicates whether this endpoint is the default endpoint for the associated DB proxy.
     */
    readonly isDefault: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A mapping of tags to assign to the resource.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Indicates whether the DB proxy endpoint can be used for read/write or read-only operations. The default is `READ_WRITE`. Valid values are `READ_WRITE` and `READ_ONLY`.
     */
    readonly targetRole: pulumi.Output<string | undefined>;
    /**
     * The VPC ID of the DB proxy endpoint.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * One or more VPC security group IDs to associate with the new proxy.
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[]>;
    /**
     * One or more VPC subnet IDs to associate with the new proxy.
     */
    readonly vpcSubnetIds: pulumi.Output<string[]>;
    /**
     * Create a ProxyEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProxyEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProxyEndpoint resources.
 */
export interface ProxyEndpointState {
    /**
     * The Amazon Resource Name (ARN) for the proxy endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * The identifier for the proxy endpoint. An identifier must begin with a letter and must contain only ASCII letters, digits, and hyphens; it can't end with a hyphen or contain two consecutive hyphens.
     */
    dbProxyEndpointName?: pulumi.Input<string>;
    /**
     * The name of the DB proxy associated with the DB proxy endpoint that you create.
     */
    dbProxyName?: pulumi.Input<string>;
    /**
     * The endpoint that you can use to connect to the proxy. You include the endpoint value in the connection string for a database client application.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Indicates whether this endpoint is the default endpoint for the associated DB proxy.
     */
    isDefault?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Indicates whether the DB proxy endpoint can be used for read/write or read-only operations. The default is `READ_WRITE`. Valid values are `READ_WRITE` and `READ_ONLY`.
     */
    targetRole?: pulumi.Input<string>;
    /**
     * The VPC ID of the DB proxy endpoint.
     */
    vpcId?: pulumi.Input<string>;
    /**
     * One or more VPC security group IDs to associate with the new proxy.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more VPC subnet IDs to associate with the new proxy.
     */
    vpcSubnetIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a ProxyEndpoint resource.
 */
export interface ProxyEndpointArgs {
    /**
     * The identifier for the proxy endpoint. An identifier must begin with a letter and must contain only ASCII letters, digits, and hyphens; it can't end with a hyphen or contain two consecutive hyphens.
     */
    dbProxyEndpointName: pulumi.Input<string>;
    /**
     * The name of the DB proxy associated with the DB proxy endpoint that you create.
     */
    dbProxyName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Indicates whether the DB proxy endpoint can be used for read/write or read-only operations. The default is `READ_WRITE`. Valid values are `READ_WRITE` and `READ_ONLY`.
     */
    targetRole?: pulumi.Input<string>;
    /**
     * One or more VPC security group IDs to associate with the new proxy.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more VPC subnet IDs to associate with the new proxy.
     */
    vpcSubnetIds: pulumi.Input<pulumi.Input<string>[]>;
}
