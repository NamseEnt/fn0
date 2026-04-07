import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a WAF Regional XSS Match Set Resource for use with Application Load Balancer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const xssMatchSet = new aws.wafregional.XssMatchSet("xss_match_set", {
 *     name: "xss_match_set",
 *     xssMatchTuples: [
 *         {
 *             textTransformation: "NONE",
 *             fieldToMatch: {
 *                 type: "URI",
 *             },
 *         },
 *         {
 *             textTransformation: "NONE",
 *             fieldToMatch: {
 *                 type: "QUERY_STRING",
 *             },
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS WAF Regional XSS Match using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/xssMatchSet:XssMatchSet example 12345abcde
 * ```
 */
export declare class XssMatchSet extends pulumi.CustomResource {
    /**
     * Get an existing XssMatchSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: XssMatchSetState, opts?: pulumi.CustomResourceOptions): XssMatchSet;
    /**
     * Returns true if the given object is an instance of XssMatchSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is XssMatchSet;
    /**
     * The name of the set
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The parts of web requests that you want to inspect for cross-site scripting attacks.
     */
    readonly xssMatchTuples: pulumi.Output<outputs.wafregional.XssMatchSetXssMatchTuple[] | undefined>;
    /**
     * Create a XssMatchSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: XssMatchSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering XssMatchSet resources.
 */
export interface XssMatchSetState {
    /**
     * The name of the set
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The parts of web requests that you want to inspect for cross-site scripting attacks.
     */
    xssMatchTuples?: pulumi.Input<pulumi.Input<inputs.wafregional.XssMatchSetXssMatchTuple>[]>;
}
/**
 * The set of arguments for constructing a XssMatchSet resource.
 */
export interface XssMatchSetArgs {
    /**
     * The name of the set
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The parts of web requests that you want to inspect for cross-site scripting attacks.
     */
    xssMatchTuples?: pulumi.Input<pulumi.Input<inputs.wafregional.XssMatchSetXssMatchTuple>[]>;
}
