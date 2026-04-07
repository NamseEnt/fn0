import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a WAF Regional Byte Match Set Resource for use with Application Load Balancer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const byteSet = new aws.wafregional.ByteMatchSet("byte_set", {
 *     name: "my_waf_byte_match_set",
 *     byteMatchTuples: [{
 *         textTransformation: "NONE",
 *         targetString: "badrefer1",
 *         positionalConstraint: "CONTAINS",
 *         fieldToMatch: {
 *             type: "HEADER",
 *             data: "referer",
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WAF Regional Byte Match Set using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/byteMatchSet:ByteMatchSet byte_set a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
 * ```
 */
export declare class ByteMatchSet extends pulumi.CustomResource {
    /**
     * Get an existing ByteMatchSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ByteMatchSetState, opts?: pulumi.CustomResourceOptions): ByteMatchSet;
    /**
     * Returns true if the given object is an instance of ByteMatchSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ByteMatchSet;
    /**
     * Settings for the ByteMatchSet, such as the bytes (typically a string that corresponds with ASCII characters) that you want AWS WAF to search for in web requests. ByteMatchTuple documented below.
     */
    readonly byteMatchTuples: pulumi.Output<outputs.wafregional.ByteMatchSetByteMatchTuple[] | undefined>;
    /**
     * The name or description of the ByteMatchSet.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ByteMatchSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ByteMatchSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ByteMatchSet resources.
 */
export interface ByteMatchSetState {
    /**
     * Settings for the ByteMatchSet, such as the bytes (typically a string that corresponds with ASCII characters) that you want AWS WAF to search for in web requests. ByteMatchTuple documented below.
     */
    byteMatchTuples?: pulumi.Input<pulumi.Input<inputs.wafregional.ByteMatchSetByteMatchTuple>[]>;
    /**
     * The name or description of the ByteMatchSet.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ByteMatchSet resource.
 */
export interface ByteMatchSetArgs {
    /**
     * Settings for the ByteMatchSet, such as the bytes (typically a string that corresponds with ASCII characters) that you want AWS WAF to search for in web requests. ByteMatchTuple documented below.
     */
    byteMatchTuples?: pulumi.Input<pulumi.Input<inputs.wafregional.ByteMatchSetByteMatchTuple>[]>;
    /**
     * The name or description of the ByteMatchSet.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
