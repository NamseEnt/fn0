import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a DAX Parameter Group resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dax.ParameterGroup("example", {
 *     name: "example",
 *     parameters: [
 *         {
 *             name: "query-ttl-millis",
 *             value: "100000",
 *         },
 *         {
 *             name: "record-ttl-millis",
 *             value: "100000",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DAX Parameter Group using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:dax/parameterGroup:ParameterGroup example my_dax_pg
 * ```
 */
export declare class ParameterGroup extends pulumi.CustomResource {
    /**
     * Get an existing ParameterGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ParameterGroupState, opts?: pulumi.CustomResourceOptions): ParameterGroup;
    /**
     * Returns true if the given object is an instance of ParameterGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ParameterGroup;
    /**
     * A description of the parameter group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the parameter group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The parameters of the parameter group.
     */
    readonly parameters: pulumi.Output<outputs.dax.ParameterGroupParameter[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ParameterGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ParameterGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ParameterGroup resources.
 */
export interface ParameterGroupState {
    /**
     * A description of the parameter group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the parameter group.
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters of the parameter group.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.dax.ParameterGroupParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ParameterGroup resource.
 */
export interface ParameterGroupArgs {
    /**
     * A description of the parameter group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the parameter group.
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters of the parameter group.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.dax.ParameterGroupParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
