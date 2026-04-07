import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Global Accelerator custom routing endpoint group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.globalaccelerator.CustomRoutingEndpointGroup("example", {
 *     listenerArn: exampleAwsGlobalacceleratorCustomRoutingListener.arn,
 *     destinationConfigurations: [{
 *         fromPort: 80,
 *         toPort: 8080,
 *         protocols: ["TCP"],
 *     }],
 *     endpointConfigurations: [{
 *         endpointId: exampleAwsSubnet.id,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Global Accelerator custom routing endpoint group.
 *
 * Using `pulumi import`, import Global Accelerator custom routing endpoint groups using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_globalaccelerator_custom_routing_endpoint_group.example arn:aws:globalaccelerator::111111111111:accelerator/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx/listener/xxxxxxx/endpoint-group/xxxxxxxx
 */
export declare class CustomRoutingEndpointGroup extends pulumi.CustomResource {
    /**
     * Get an existing CustomRoutingEndpointGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomRoutingEndpointGroupState, opts?: pulumi.CustomResourceOptions): CustomRoutingEndpointGroup;
    /**
     * Returns true if the given object is an instance of CustomRoutingEndpointGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomRoutingEndpointGroup;
    /**
     * The Amazon Resource Name (ARN) of the custom routing endpoint group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The port ranges and protocols for all endpoints in a custom routing endpoint group to accept client traffic on. Fields documented below.
     */
    readonly destinationConfigurations: pulumi.Output<outputs.globalaccelerator.CustomRoutingEndpointGroupDestinationConfiguration[]>;
    /**
     * The list of endpoint objects. Fields documented below.
     */
    readonly endpointConfigurations: pulumi.Output<outputs.globalaccelerator.CustomRoutingEndpointGroupEndpointConfiguration[] | undefined>;
    /**
     * The name of the AWS Region where the custom routing endpoint group is located.
     */
    readonly endpointGroupRegion: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the custom routing listener.
     */
    readonly listenerArn: pulumi.Output<string>;
    /**
     * Create a CustomRoutingEndpointGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomRoutingEndpointGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomRoutingEndpointGroup resources.
 */
export interface CustomRoutingEndpointGroupState {
    /**
     * The Amazon Resource Name (ARN) of the custom routing endpoint group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The port ranges and protocols for all endpoints in a custom routing endpoint group to accept client traffic on. Fields documented below.
     */
    destinationConfigurations?: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CustomRoutingEndpointGroupDestinationConfiguration>[]>;
    /**
     * The list of endpoint objects. Fields documented below.
     */
    endpointConfigurations?: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CustomRoutingEndpointGroupEndpointConfiguration>[]>;
    /**
     * The name of the AWS Region where the custom routing endpoint group is located.
     */
    endpointGroupRegion?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the custom routing listener.
     */
    listenerArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CustomRoutingEndpointGroup resource.
 */
export interface CustomRoutingEndpointGroupArgs {
    /**
     * The port ranges and protocols for all endpoints in a custom routing endpoint group to accept client traffic on. Fields documented below.
     */
    destinationConfigurations: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CustomRoutingEndpointGroupDestinationConfiguration>[]>;
    /**
     * The list of endpoint objects. Fields documented below.
     */
    endpointConfigurations?: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CustomRoutingEndpointGroupEndpointConfiguration>[]>;
    /**
     * The name of the AWS Region where the custom routing endpoint group is located.
     */
    endpointGroupRegion?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the custom routing listener.
     */
    listenerArn: pulumi.Input<string>;
}
