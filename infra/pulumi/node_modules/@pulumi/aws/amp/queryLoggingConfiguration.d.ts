import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon Managed Service for Prometheus (AMP) Query Logging Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amp.Workspace("example", {alias: "example"});
 * const exampleLogGroup = new aws.cloudwatch.LogGroup("example", {name: "/aws/prometheus/query-logs/example"});
 * const exampleQueryLoggingConfiguration = new aws.amp.QueryLoggingConfiguration("example", {
 *     workspaceId: example.id,
 *     destinations: [{
 *         cloudwatchLogs: {
 *             logGroupArn: pulumi.interpolate`${exampleLogGroup.arn}:*`,
 *         },
 *         filters: {
 *             qspThreshold: 1000,
 *         },
 *     }],
 * });
 * ```
 */
export declare class QueryLoggingConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing QueryLoggingConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: QueryLoggingConfigurationState, opts?: pulumi.CustomResourceOptions): QueryLoggingConfiguration;
    /**
     * Returns true if the given object is an instance of QueryLoggingConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is QueryLoggingConfiguration;
    /**
     * Configuration block for the logging destinations. See `destinations`.
     */
    readonly destinations: pulumi.Output<outputs.amp.QueryLoggingConfigurationDestination[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.amp.QueryLoggingConfigurationTimeouts | undefined>;
    /**
     * The ID of the AMP workspace for which to configure query logging.
     *
     * The following arguments are optional:
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a QueryLoggingConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: QueryLoggingConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering QueryLoggingConfiguration resources.
 */
export interface QueryLoggingConfigurationState {
    /**
     * Configuration block for the logging destinations. See `destinations`.
     */
    destinations?: pulumi.Input<pulumi.Input<inputs.amp.QueryLoggingConfigurationDestination>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.amp.QueryLoggingConfigurationTimeouts>;
    /**
     * The ID of the AMP workspace for which to configure query logging.
     *
     * The following arguments are optional:
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a QueryLoggingConfiguration resource.
 */
export interface QueryLoggingConfigurationArgs {
    /**
     * Configuration block for the logging destinations. See `destinations`.
     */
    destinations?: pulumi.Input<pulumi.Input<inputs.amp.QueryLoggingConfigurationDestination>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.amp.QueryLoggingConfigurationTimeouts>;
    /**
     * The ID of the AMP workspace for which to configure query logging.
     *
     * The following arguments are optional:
     */
    workspaceId: pulumi.Input<string>;
}
