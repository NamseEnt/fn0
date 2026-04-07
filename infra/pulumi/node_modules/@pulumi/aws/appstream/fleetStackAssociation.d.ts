import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AppStream Fleet Stack association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appstream.Fleet("example", {
 *     name: "NAME",
 *     imageName: "Amazon-AppStream2-Sample-Image-03-11-2023",
 *     instanceType: "stream.standard.small",
 *     computeCapacity: {
 *         desiredInstances: 1,
 *     },
 * });
 * const exampleStack = new aws.appstream.Stack("example", {name: "STACK NAME"});
 * const exampleFleetStackAssociation = new aws.appstream.FleetStackAssociation("example", {
 *     fleetName: example.name,
 *     stackName: exampleStack.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppStream Stack Fleet Association using the `fleet_name` and `stack_name` separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:appstream/fleetStackAssociation:FleetStackAssociation example fleetName/stackName
 * ```
 */
export declare class FleetStackAssociation extends pulumi.CustomResource {
    /**
     * Get an existing FleetStackAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FleetStackAssociationState, opts?: pulumi.CustomResourceOptions): FleetStackAssociation;
    /**
     * Returns true if the given object is an instance of FleetStackAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FleetStackAssociation;
    /**
     * Name of the fleet.
     */
    readonly fleetName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the stack.
     */
    readonly stackName: pulumi.Output<string>;
    /**
     * Create a FleetStackAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FleetStackAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FleetStackAssociation resources.
 */
export interface FleetStackAssociationState {
    /**
     * Name of the fleet.
     */
    fleetName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the stack.
     */
    stackName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FleetStackAssociation resource.
 */
export interface FleetStackAssociationArgs {
    /**
     * Name of the fleet.
     */
    fleetName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the stack.
     */
    stackName: pulumi.Input<string>;
}
