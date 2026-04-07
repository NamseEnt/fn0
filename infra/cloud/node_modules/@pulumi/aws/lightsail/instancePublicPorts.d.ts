import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages public ports for a Lightsail instance. Use this resource to open ports for a specific Amazon Lightsail instance and specify the IP addresses allowed to connect to the instance through the ports and the protocol.
 *
 * > See [What is Amazon Lightsail?](https://lightsail.aws.amazon.com/ls/docs/getting-started/article/what-is-amazon-lightsail) for more information.
 *
 * > **Note:** Lightsail is currently only supported in a limited number of AWS Regions, please see ["Regions and Availability Zones in Amazon Lightsail"](https://lightsail.aws.amazon.com/ls/docs/overview/article/understanding-regions-and-availability-zones-in-amazon-lightsail) for more details.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["opt-in-not-required"],
 *     }],
 * });
 * const example = new aws.lightsail.Instance("example", {
 *     name: "example-instance",
 *     availabilityZone: available.then(available => available.names?.[0]),
 *     blueprintId: "amazon_linux_2",
 *     bundleId: "nano_3_0",
 * });
 * const exampleInstancePublicPorts = new aws.lightsail.InstancePublicPorts("example", {
 *     instanceName: example.name,
 *     portInfos: [
 *         {
 *             protocol: "tcp",
 *             fromPort: 80,
 *             toPort: 80,
 *         },
 *         {
 *             protocol: "tcp",
 *             fromPort: 443,
 *             toPort: 443,
 *             cidrs: ["192.168.1.0/24"],
 *         },
 *     ],
 * });
 * ```
 */
export declare class InstancePublicPorts extends pulumi.CustomResource {
    /**
     * Get an existing InstancePublicPorts resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstancePublicPortsState, opts?: pulumi.CustomResourceOptions): InstancePublicPorts;
    /**
     * Returns true if the given object is an instance of InstancePublicPorts.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstancePublicPorts;
    /**
     * Name of the instance for which to open ports.
     */
    readonly instanceName: pulumi.Output<string>;
    /**
     * Descriptor of the ports to open for the specified instance. AWS closes all currently open ports that are not included in this argument. See `portInfo` Block for details.
     *
     * The following arguments are optional:
     */
    readonly portInfos: pulumi.Output<outputs.lightsail.InstancePublicPortsPortInfo[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a InstancePublicPorts resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstancePublicPortsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstancePublicPorts resources.
 */
export interface InstancePublicPortsState {
    /**
     * Name of the instance for which to open ports.
     */
    instanceName?: pulumi.Input<string>;
    /**
     * Descriptor of the ports to open for the specified instance. AWS closes all currently open ports that are not included in this argument. See `portInfo` Block for details.
     *
     * The following arguments are optional:
     */
    portInfos?: pulumi.Input<pulumi.Input<inputs.lightsail.InstancePublicPortsPortInfo>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InstancePublicPorts resource.
 */
export interface InstancePublicPortsArgs {
    /**
     * Name of the instance for which to open ports.
     */
    instanceName: pulumi.Input<string>;
    /**
     * Descriptor of the ports to open for the specified instance. AWS closes all currently open ports that are not included in this argument. See `portInfo` Block for details.
     *
     * The following arguments are optional:
     */
    portInfos: pulumi.Input<pulumi.Input<inputs.lightsail.InstancePublicPortsPortInfo>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
