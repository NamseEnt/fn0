import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a confirmation of the creation of the specified hosted connection on an interconnect.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const confirmation = new aws.directconnect.ConnectionConfirmation("confirmation", {connectionId: "dxcon-ffabc123"});
 * ```
 */
export declare class ConnectionConfirmation extends pulumi.CustomResource {
    /**
     * Get an existing ConnectionConfirmation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectionConfirmationState, opts?: pulumi.CustomResourceOptions): ConnectionConfirmation;
    /**
     * Returns true if the given object is an instance of ConnectionConfirmation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConnectionConfirmation;
    /**
     * The ID of the hosted connection.
     */
    readonly connectionId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ConnectionConfirmation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectionConfirmationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConnectionConfirmation resources.
 */
export interface ConnectionConfirmationState {
    /**
     * The ID of the hosted connection.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConnectionConfirmation resource.
 */
export interface ConnectionConfirmationArgs {
    /**
     * The ID of the hosted connection.
     */
    connectionId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
