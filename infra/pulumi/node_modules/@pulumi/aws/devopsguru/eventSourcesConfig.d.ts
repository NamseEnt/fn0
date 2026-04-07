import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devopsguru.EventSourcesConfig("example", {eventSources: [{
 *     amazonCodeGuruProfilers: [{
 *         status: "ENABLED",
 *     }],
 * }]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DevOps Guru Event Sources Config using the region. For example:
 *
 * ```sh
 * $ pulumi import aws:devopsguru/eventSourcesConfig:EventSourcesConfig example us-east-1
 * ```
 */
export declare class EventSourcesConfig extends pulumi.CustomResource {
    /**
     * Get an existing EventSourcesConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventSourcesConfigState, opts?: pulumi.CustomResourceOptions): EventSourcesConfig;
    /**
     * Returns true if the given object is an instance of EventSourcesConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventSourcesConfig;
    /**
     * Configuration information about the integration of DevOps Guru as the Consumer via EventBridge with another AWS Service. See `eventSources` below.
     */
    readonly eventSources: pulumi.Output<outputs.devopsguru.EventSourcesConfigEventSource[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a EventSourcesConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: EventSourcesConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventSourcesConfig resources.
 */
export interface EventSourcesConfigState {
    /**
     * Configuration information about the integration of DevOps Guru as the Consumer via EventBridge with another AWS Service. See `eventSources` below.
     */
    eventSources?: pulumi.Input<pulumi.Input<inputs.devopsguru.EventSourcesConfigEventSource>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventSourcesConfig resource.
 */
export interface EventSourcesConfigArgs {
    /**
     * Configuration information about the integration of DevOps Guru as the Consumer via EventBridge with another AWS Service. See `eventSources` below.
     */
    eventSources?: pulumi.Input<pulumi.Input<inputs.devopsguru.EventSourcesConfigEventSource>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
