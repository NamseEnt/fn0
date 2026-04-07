import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to create an EventBridge Global Endpoint.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _this = new aws.cloudwatch.EventEndpoint("this", {
 *     name: "global-endpoint",
 *     roleArn: replication.arn,
 *     eventBuses: [
 *         {
 *             eventBusArn: primary.arn,
 *         },
 *         {
 *             eventBusArn: secondary.arn,
 *         },
 *     ],
 *     replicationConfig: {
 *         state: "DISABLED",
 *     },
 *     routingConfig: {
 *         failoverConfig: {
 *             primary: {
 *                 healthCheck: primaryAwsRoute53HealthCheck.arn,
 *             },
 *             secondary: {
 *                 route: "us-east-2",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EventBridge Global Endpoints using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/eventEndpoint:EventEndpoint imported_endpoint example-endpoint
 * ```
 */
export declare class EventEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing EventEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventEndpointState, opts?: pulumi.CustomResourceOptions): EventEndpoint;
    /**
     * Returns true if the given object is an instance of EventEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventEndpoint;
    /**
     * The ARN of the endpoint that was created.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the global endpoint.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The URL of the endpoint that was created.
     */
    readonly endpointUrl: pulumi.Output<string>;
    /**
     * The event buses to use. The names of the event buses must be identical in each Region. Exactly two event buses are required. Documented below.
     */
    readonly eventBuses: pulumi.Output<outputs.cloudwatch.EventEndpointEventBus[]>;
    /**
     * The name of the global endpoint.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Parameters used for replication. Documented below.
     */
    readonly replicationConfig: pulumi.Output<outputs.cloudwatch.EventEndpointReplicationConfig | undefined>;
    /**
     * The ARN of the IAM role used for replication between event buses.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    /**
     * Parameters used for routing, including the health check and secondary Region. Documented below.
     */
    readonly routingConfig: pulumi.Output<outputs.cloudwatch.EventEndpointRoutingConfig>;
    /**
     * Create a EventEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventEndpoint resources.
 */
export interface EventEndpointState {
    /**
     * The ARN of the endpoint that was created.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the global endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * The URL of the endpoint that was created.
     */
    endpointUrl?: pulumi.Input<string>;
    /**
     * The event buses to use. The names of the event buses must be identical in each Region. Exactly two event buses are required. Documented below.
     */
    eventBuses?: pulumi.Input<pulumi.Input<inputs.cloudwatch.EventEndpointEventBus>[]>;
    /**
     * The name of the global endpoint.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Parameters used for replication. Documented below.
     */
    replicationConfig?: pulumi.Input<inputs.cloudwatch.EventEndpointReplicationConfig>;
    /**
     * The ARN of the IAM role used for replication between event buses.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Parameters used for routing, including the health check and secondary Region. Documented below.
     */
    routingConfig?: pulumi.Input<inputs.cloudwatch.EventEndpointRoutingConfig>;
}
/**
 * The set of arguments for constructing a EventEndpoint resource.
 */
export interface EventEndpointArgs {
    /**
     * A description of the global endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * The event buses to use. The names of the event buses must be identical in each Region. Exactly two event buses are required. Documented below.
     */
    eventBuses: pulumi.Input<pulumi.Input<inputs.cloudwatch.EventEndpointEventBus>[]>;
    /**
     * The name of the global endpoint.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Parameters used for replication. Documented below.
     */
    replicationConfig?: pulumi.Input<inputs.cloudwatch.EventEndpointReplicationConfig>;
    /**
     * The ARN of the IAM role used for replication between event buses.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Parameters used for routing, including the health check and secondary Region. Documented below.
     */
    routingConfig: pulumi.Input<inputs.cloudwatch.EventEndpointRoutingConfig>;
}
