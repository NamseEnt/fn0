import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route53 CIDR collection resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53.CidrCollection("example", {name: "collection-1"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CIDR collections using their ID. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/cidrCollection:CidrCollection example 9ac32814-3e67-0932-6048-8d779cc6f511
 * ```
 */
export declare class CidrCollection extends pulumi.CustomResource {
    /**
     * Get an existing CidrCollection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CidrCollectionState, opts?: pulumi.CustomResourceOptions): CidrCollection;
    /**
     * Returns true if the given object is an instance of CidrCollection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CidrCollection;
    /**
     * The Amazon Resource Name (ARN) of the CIDR collection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Unique name for the CIDR collection.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The lastest version of the CIDR collection.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a CidrCollection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: CidrCollectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CidrCollection resources.
 */
export interface CidrCollectionState {
    /**
     * The Amazon Resource Name (ARN) of the CIDR collection.
     */
    arn?: pulumi.Input<string>;
    /**
     * Unique name for the CIDR collection.
     */
    name?: pulumi.Input<string>;
    /**
     * The lastest version of the CIDR collection.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a CidrCollection resource.
 */
export interface CidrCollectionArgs {
    /**
     * Unique name for the CIDR collection.
     */
    name?: pulumi.Input<string>;
}
