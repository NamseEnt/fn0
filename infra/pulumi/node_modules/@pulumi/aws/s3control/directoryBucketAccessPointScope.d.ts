import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### S3 Access Point Scope for a directory bucket in an AWS Local Zone
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 * });
 * const example = new aws.s3.DirectoryBucket("example", {
 *     bucket: "example--zoneId--x-s3",
 *     location: {
 *         name: available.then(available => available.zoneIds?.[0]),
 *     },
 * });
 * const exampleAccessPoint = new aws.s3.AccessPoint("example", {
 *     bucket: example.id,
 *     name: "example--zoneId--xa-s3",
 * });
 * const exampleDirectoryBucketAccessPointScope = new aws.s3control.DirectoryBucketAccessPointScope("example", {
 *     name: "example--zoneId--xa-s3",
 *     accountId: "123456789012",
 *     scope: {
 *         permissions: [
 *             "GetObject",
 *             "ListBucket",
 *         ],
 *         prefixes: [
 *             "myobject1.csv",
 *             "myobject2*",
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Access Point Scope using access point name and AWS account ID separated by a colon (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/directoryBucketAccessPointScope:DirectoryBucketAccessPointScope example example--zoneid--xa-s3,123456789012
 * ```
 */
export declare class DirectoryBucketAccessPointScope extends pulumi.CustomResource {
    /**
     * Get an existing DirectoryBucketAccessPointScope resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DirectoryBucketAccessPointScopeState, opts?: pulumi.CustomResourceOptions): DirectoryBucketAccessPointScope;
    /**
     * Returns true if the given object is an instance of DirectoryBucketAccessPointScope.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DirectoryBucketAccessPointScope;
    /**
     * The AWS account ID that owns the specified access point.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The name of the access point that you want to apply the scope to.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * . Scope is used to restrict access to specific prefixes, API operations, or a combination of both. To remove the `scope`, set it to `{permissions=[] prefixes=[]}`. The default scope is `{permissions=[] prefixes=[]}`.
     */
    readonly scope: pulumi.Output<outputs.s3control.DirectoryBucketAccessPointScopeScope | undefined>;
    /**
     * Create a DirectoryBucketAccessPointScope resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DirectoryBucketAccessPointScopeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DirectoryBucketAccessPointScope resources.
 */
export interface DirectoryBucketAccessPointScopeState {
    /**
     * The AWS account ID that owns the specified access point.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The name of the access point that you want to apply the scope to.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * . Scope is used to restrict access to specific prefixes, API operations, or a combination of both. To remove the `scope`, set it to `{permissions=[] prefixes=[]}`. The default scope is `{permissions=[] prefixes=[]}`.
     */
    scope?: pulumi.Input<inputs.s3control.DirectoryBucketAccessPointScopeScope>;
}
/**
 * The set of arguments for constructing a DirectoryBucketAccessPointScope resource.
 */
export interface DirectoryBucketAccessPointScopeArgs {
    /**
     * The AWS account ID that owns the specified access point.
     */
    accountId: pulumi.Input<string>;
    /**
     * The name of the access point that you want to apply the scope to.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * . Scope is used to restrict access to specific prefixes, API operations, or a combination of both. To remove the `scope`, set it to `{permissions=[] prefixes=[]}`. The default scope is `{permissions=[] prefixes=[]}`.
     */
    scope?: pulumi.Input<inputs.s3control.DirectoryBucketAccessPointScopeScope>;
}
