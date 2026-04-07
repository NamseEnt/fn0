import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Global Accelerator custom routing listener.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.globalaccelerator.CustomRoutingAccelerator("example", {
 *     name: "Example",
 *     ipAddressType: "IPV4",
 *     enabled: true,
 *     attributes: {
 *         flowLogsEnabled: true,
 *         flowLogsS3Bucket: "example-bucket",
 *         flowLogsS3Prefix: "flow-logs/",
 *     },
 * });
 * const exampleCustomRoutingListener = new aws.globalaccelerator.CustomRoutingListener("example", {
 *     acceleratorArn: example.arn,
 *     portRanges: [{
 *         fromPort: 80,
 *         toPort: 80,
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
 * - `arn` (String) Amazon Resource Name (ARN) of the Global Accelerator custom routing listener.
 *
 * Using `pulumi import`, import Global Accelerator custom routing listeners using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_globalaccelerator_custom_routing_listener.example arn:aws:globalaccelerator::111111111111:accelerator/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx/listener/xxxxxxxx
 */
export declare class CustomRoutingListener extends pulumi.CustomResource {
    /**
     * Get an existing CustomRoutingListener resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomRoutingListenerState, opts?: pulumi.CustomResourceOptions): CustomRoutingListener;
    /**
     * Returns true if the given object is an instance of CustomRoutingListener.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomRoutingListener;
    /**
     * The Amazon Resource Name (ARN) of a custom routing accelerator.
     */
    readonly acceleratorArn: pulumi.Output<string>;
    readonly arn: pulumi.Output<string>;
    /**
     * The list of port ranges for the connections from clients to the accelerator. Fields documented below.
     */
    readonly portRanges: pulumi.Output<outputs.globalaccelerator.CustomRoutingListenerPortRange[]>;
    /**
     * Create a CustomRoutingListener resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomRoutingListenerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomRoutingListener resources.
 */
export interface CustomRoutingListenerState {
    /**
     * The Amazon Resource Name (ARN) of a custom routing accelerator.
     */
    acceleratorArn?: pulumi.Input<string>;
    arn?: pulumi.Input<string>;
    /**
     * The list of port ranges for the connections from clients to the accelerator. Fields documented below.
     */
    portRanges?: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CustomRoutingListenerPortRange>[]>;
}
/**
 * The set of arguments for constructing a CustomRoutingListener resource.
 */
export interface CustomRoutingListenerArgs {
    /**
     * The Amazon Resource Name (ARN) of a custom routing accelerator.
     */
    acceleratorArn: pulumi.Input<string>;
    /**
     * The list of port ranges for the connections from clients to the accelerator. Fields documented below.
     */
    portRanges: pulumi.Input<pulumi.Input<inputs.globalaccelerator.CustomRoutingListenerPortRange>[]>;
}
