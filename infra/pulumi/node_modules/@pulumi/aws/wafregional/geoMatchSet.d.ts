import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a WAF Regional Geo Match Set Resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const geoMatchSet = new aws.wafregional.GeoMatchSet("geo_match_set", {
 *     name: "geo_match_set",
 *     geoMatchConstraints: [
 *         {
 *             type: "Country",
 *             value: "US",
 *         },
 *         {
 *             type: "Country",
 *             value: "CA",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WAF Regional Geo Match Set using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/geoMatchSet:GeoMatchSet geo_match_set a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
 * ```
 */
export declare class GeoMatchSet extends pulumi.CustomResource {
    /**
     * Get an existing GeoMatchSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GeoMatchSetState, opts?: pulumi.CustomResourceOptions): GeoMatchSet;
    /**
     * Returns true if the given object is an instance of GeoMatchSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GeoMatchSet;
    /**
     * The Geo Match Constraint objects which contain the country that you want AWS WAF to search for.
     */
    readonly geoMatchConstraints: pulumi.Output<outputs.wafregional.GeoMatchSetGeoMatchConstraint[] | undefined>;
    /**
     * The name or description of the Geo Match Set.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a GeoMatchSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: GeoMatchSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GeoMatchSet resources.
 */
export interface GeoMatchSetState {
    /**
     * The Geo Match Constraint objects which contain the country that you want AWS WAF to search for.
     */
    geoMatchConstraints?: pulumi.Input<pulumi.Input<inputs.wafregional.GeoMatchSetGeoMatchConstraint>[]>;
    /**
     * The name or description of the Geo Match Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GeoMatchSet resource.
 */
export interface GeoMatchSetArgs {
    /**
     * The Geo Match Constraint objects which contain the country that you want AWS WAF to search for.
     */
    geoMatchConstraints?: pulumi.Input<pulumi.Input<inputs.wafregional.GeoMatchSetGeoMatchConstraint>[]>;
    /**
     * The name or description of the Geo Match Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
