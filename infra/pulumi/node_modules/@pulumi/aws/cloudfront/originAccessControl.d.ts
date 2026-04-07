import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS CloudFront Origin Access Control, which is used by CloudFront Distributions with an Amazon S3 bucket as the origin.
 *
 * Read more about Origin Access Control in the [CloudFront Developer Guide](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudfront.OriginAccessControl("example", {
 *     name: "example",
 *     description: "Example Policy",
 *     originAccessControlOriginType: "s3",
 *     signingBehavior: "always",
 *     signingProtocol: "sigv4",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudFront Origin Access Control using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/originAccessControl:OriginAccessControl example E327GJI25M56DG
 * ```
 */
export declare class OriginAccessControl extends pulumi.CustomResource {
    /**
     * Get an existing OriginAccessControl resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OriginAccessControlState, opts?: pulumi.CustomResourceOptions): OriginAccessControl;
    /**
     * Returns true if the given object is an instance of OriginAccessControl.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OriginAccessControl;
    /**
     * The Origin Access Control ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the Origin Access Control. Defaults to "Managed by Pulumi" if omitted.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The current version of this Origin Access Control.
     */
    readonly etag: pulumi.Output<string>;
    /**
     * A name that identifies the Origin Access Control.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The type of origin that this Origin Access Control is for. Valid values are `lambda`, `mediapackagev2`, `mediastore`, and `s3`.
     */
    readonly originAccessControlOriginType: pulumi.Output<string>;
    /**
     * Specifies which requests CloudFront signs. Specify `always` for the most common use case. Allowed values: `always`, `never`, and `no-override`.
     */
    readonly signingBehavior: pulumi.Output<string>;
    /**
     * Determines how CloudFront signs (authenticates) requests. The only valid value is `sigv4`.
     */
    readonly signingProtocol: pulumi.Output<string>;
    /**
     * Create a OriginAccessControl resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OriginAccessControlArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OriginAccessControl resources.
 */
export interface OriginAccessControlState {
    /**
     * The Origin Access Control ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the Origin Access Control. Defaults to "Managed by Pulumi" if omitted.
     */
    description?: pulumi.Input<string>;
    /**
     * The current version of this Origin Access Control.
     */
    etag?: pulumi.Input<string>;
    /**
     * A name that identifies the Origin Access Control.
     */
    name?: pulumi.Input<string>;
    /**
     * The type of origin that this Origin Access Control is for. Valid values are `lambda`, `mediapackagev2`, `mediastore`, and `s3`.
     */
    originAccessControlOriginType?: pulumi.Input<string>;
    /**
     * Specifies which requests CloudFront signs. Specify `always` for the most common use case. Allowed values: `always`, `never`, and `no-override`.
     */
    signingBehavior?: pulumi.Input<string>;
    /**
     * Determines how CloudFront signs (authenticates) requests. The only valid value is `sigv4`.
     */
    signingProtocol?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OriginAccessControl resource.
 */
export interface OriginAccessControlArgs {
    /**
     * The description of the Origin Access Control. Defaults to "Managed by Pulumi" if omitted.
     */
    description?: pulumi.Input<string>;
    /**
     * A name that identifies the Origin Access Control.
     */
    name?: pulumi.Input<string>;
    /**
     * The type of origin that this Origin Access Control is for. Valid values are `lambda`, `mediapackagev2`, `mediastore`, and `s3`.
     */
    originAccessControlOriginType: pulumi.Input<string>;
    /**
     * Specifies which requests CloudFront signs. Specify `always` for the most common use case. Allowed values: `always`, `never`, and `no-override`.
     */
    signingBehavior: pulumi.Input<string>;
    /**
     * Determines how CloudFront signs (authenticates) requests. The only valid value is `sigv4`.
     */
    signingProtocol: pulumi.Input<string>;
}
