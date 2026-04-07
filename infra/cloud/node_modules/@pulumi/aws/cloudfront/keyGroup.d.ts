import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * The following example below creates a CloudFront key group.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.cloudfront.PublicKey("example", {
 *     comment: "example public key",
 *     encodedKey: std.file({
 *         input: "public_key.pem",
 *     }).then(invoke => invoke.result),
 *     name: "example-key",
 * });
 * const exampleKeyGroup = new aws.cloudfront.KeyGroup("example", {
 *     comment: "example key group",
 *     items: [example.id],
 *     name: "example-key-group",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudFront Key Group using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/keyGroup:KeyGroup example 4b4f2r1c-315d-5c2e-f093-216t50jed10f
 * ```
 */
export declare class KeyGroup extends pulumi.CustomResource {
    /**
     * Get an existing KeyGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyGroupState, opts?: pulumi.CustomResourceOptions): KeyGroup;
    /**
     * Returns true if the given object is an instance of KeyGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyGroup;
    /**
     * A comment to describe the key group..
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * The identifier for this version of the key group.
     */
    readonly etag: pulumi.Output<string>;
    /**
     * A list of the identifiers of the public keys in the key group.
     */
    readonly items: pulumi.Output<string[]>;
    /**
     * A name to identify the key group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Create a KeyGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KeyGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyGroup resources.
 */
export interface KeyGroupState {
    /**
     * A comment to describe the key group..
     */
    comment?: pulumi.Input<string>;
    /**
     * The identifier for this version of the key group.
     */
    etag?: pulumi.Input<string>;
    /**
     * A list of the identifiers of the public keys in the key group.
     */
    items?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A name to identify the key group.
     */
    name?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a KeyGroup resource.
 */
export interface KeyGroupArgs {
    /**
     * A comment to describe the key group..
     */
    comment?: pulumi.Input<string>;
    /**
     * A list of the identifiers of the public keys in the key group.
     */
    items: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A name to identify the key group.
     */
    name?: pulumi.Input<string>;
}
