import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route53 CIDR location resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53.CidrCollection("example", {name: "collection-1"});
 * const exampleCidrLocation = new aws.route53.CidrLocation("example", {
 *     cidrCollectionId: example.id,
 *     name: "office",
 *     cidrBlocks: [
 *         "200.5.3.0/24",
 *         "200.6.3.0/24",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CIDR locations using their the CIDR collection ID and location name. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/cidrLocation:CidrLocation example 9ac32814-3e67-0932-6048-8d779cc6f511,office
 * ```
 */
export declare class CidrLocation extends pulumi.CustomResource {
    /**
     * Get an existing CidrLocation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CidrLocationState, opts?: pulumi.CustomResourceOptions): CidrLocation;
    /**
     * Returns true if the given object is an instance of CidrLocation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CidrLocation;
    /**
     * CIDR blocks for the location.
     */
    readonly cidrBlocks: pulumi.Output<string[]>;
    /**
     * The ID of the CIDR collection to update.
     */
    readonly cidrCollectionId: pulumi.Output<string>;
    /**
     * Name for the CIDR location.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Create a CidrLocation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CidrLocationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CidrLocation resources.
 */
export interface CidrLocationState {
    /**
     * CIDR blocks for the location.
     */
    cidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the CIDR collection to update.
     */
    cidrCollectionId?: pulumi.Input<string>;
    /**
     * Name for the CIDR location.
     */
    name?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CidrLocation resource.
 */
export interface CidrLocationArgs {
    /**
     * CIDR blocks for the location.
     */
    cidrBlocks: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the CIDR collection to update.
     */
    cidrCollectionId: pulumi.Input<string>;
    /**
     * Name for the CIDR location.
     */
    name?: pulumi.Input<string>;
}
