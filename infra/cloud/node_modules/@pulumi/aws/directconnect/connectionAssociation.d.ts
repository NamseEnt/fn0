import * as pulumi from "@pulumi/pulumi";
/**
 * Associates a Direct Connect Connection with a LAG.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Connection("example", {
 *     name: "example",
 *     bandwidth: "1Gbps",
 *     location: "EqSe2-EQ",
 * });
 * const exampleLinkAggregationGroup = new aws.directconnect.LinkAggregationGroup("example", {
 *     name: "example",
 *     connectionsBandwidth: "1Gbps",
 *     location: "EqSe2-EQ",
 * });
 * const exampleConnectionAssociation = new aws.directconnect.ConnectionAssociation("example", {
 *     connectionId: example.id,
 *     lagId: exampleLinkAggregationGroup.id,
 * });
 * ```
 */
export declare class ConnectionAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ConnectionAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectionAssociationState, opts?: pulumi.CustomResourceOptions): ConnectionAssociation;
    /**
     * Returns true if the given object is an instance of ConnectionAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConnectionAssociation;
    /**
     * The ID of the connection.
     */
    readonly connectionId: pulumi.Output<string>;
    /**
     * The ID of the LAG with which to associate the connection.
     */
    readonly lagId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ConnectionAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectionAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConnectionAssociation resources.
 */
export interface ConnectionAssociationState {
    /**
     * The ID of the connection.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * The ID of the LAG with which to associate the connection.
     */
    lagId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConnectionAssociation resource.
 */
export interface ConnectionAssociationArgs {
    /**
     * The ID of the connection.
     */
    connectionId: pulumi.Input<string>;
    /**
     * The ID of the LAG with which to associate the connection.
     */
    lagId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
