import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an S3 Access Point resource policy.
 *
 * > **NOTE on Access Points and Access Point Policies:** The provider provides both a standalone Access Point Policy resource and an Access Point resource with a resource policy defined in-line. You cannot use an Access Point with in-line resource policy in conjunction with an Access Point Policy resource. Doing so will cause a conflict of policies and will overwrite the access point's resource policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example"});
 * const exampleAccessPoint = new aws.s3.AccessPoint("example", {
 *     bucket: example.id,
 *     name: "example",
 *     publicAccessBlockConfiguration: {
 *         blockPublicAcls: true,
 *         blockPublicPolicy: false,
 *         ignorePublicAcls: true,
 *         restrictPublicBuckets: false,
 *     },
 * });
 * const exampleAccessPointPolicy = new aws.s3control.AccessPointPolicy("example", {
 *     accessPointArn: exampleAccessPoint.arn,
 *     policy: pulumi.jsonStringify({
 *         Version: "2008-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Action: "s3:GetObjectTagging",
 *             Principal: {
 *                 AWS: "*",
 *             },
 *             Resource: pulumi.interpolate`${exampleAccessPoint.arn}/object/*`,
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Access Point policies using the `access_point_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/accessPointPolicy:AccessPointPolicy example arn:aws:s3:us-west-2:123456789012:accesspoint/example
 * ```
 */
export declare class AccessPointPolicy extends pulumi.CustomResource {
    /**
     * Get an existing AccessPointPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccessPointPolicyState, opts?: pulumi.CustomResourceOptions): AccessPointPolicy;
    /**
     * Returns true if the given object is an instance of AccessPointPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccessPointPolicy;
    /**
     * The ARN of the access point that you want to associate with the specified policy.
     */
    readonly accessPointArn: pulumi.Output<string>;
    /**
     * Indicates whether this access point currently has a policy that allows public access.
     */
    readonly hasPublicAccessPolicy: pulumi.Output<boolean>;
    /**
     * The policy that you want to apply to the specified access point.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a AccessPointPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccessPointPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccessPointPolicy resources.
 */
export interface AccessPointPolicyState {
    /**
     * The ARN of the access point that you want to associate with the specified policy.
     */
    accessPointArn?: pulumi.Input<string>;
    /**
     * Indicates whether this access point currently has a policy that allows public access.
     */
    hasPublicAccessPolicy?: pulumi.Input<boolean>;
    /**
     * The policy that you want to apply to the specified access point.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccessPointPolicy resource.
 */
export interface AccessPointPolicyArgs {
    /**
     * The ARN of the access point that you want to associate with the specified policy.
     */
    accessPointArn: pulumi.Input<string>;
    /**
     * The policy that you want to apply to the specified access point.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
