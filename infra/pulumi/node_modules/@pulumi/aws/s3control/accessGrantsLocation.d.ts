import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an S3 Access Grants location.
 * A location is an S3 resource (bucket or prefix) in a permission grant that the grantee can access.
 * The S3 data must be in the same Region as your S3 Access Grants instance.
 * When you register a location, you must include the IAM role that has permission to manage the S3 location that you are registering.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3control.AccessGrantsInstance("example", {});
 * const exampleAccessGrantsLocation = new aws.s3control.AccessGrantsLocation("example", {
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     locationScope: "s3://",
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Access Grants locations using the `account_id` and `access_grants_location_id`, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/accessGrantsLocation:AccessGrantsLocation example 123456789012,default
 * ```
 */
export declare class AccessGrantsLocation extends pulumi.CustomResource {
    /**
     * Get an existing AccessGrantsLocation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccessGrantsLocationState, opts?: pulumi.CustomResourceOptions): AccessGrantsLocation;
    /**
     * Returns true if the given object is an instance of AccessGrantsLocation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccessGrantsLocation;
    /**
     * Amazon Resource Name (ARN) of the S3 Access Grants location.
     */
    readonly accessGrantsLocationArn: pulumi.Output<string>;
    /**
     * Unique ID of the S3 Access Grants location.
     */
    readonly accessGrantsLocationId: pulumi.Output<string>;
    readonly accountId: pulumi.Output<string>;
    /**
     * The ARN of the IAM role that S3 Access Grants should use when fulfilling runtime access
     * requests to the location.
     */
    readonly iamRoleArn: pulumi.Output<string>;
    /**
     * The default S3 URI `s3://` or the URI to a custom location, a specific bucket or prefix.
     */
    readonly locationScope: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a AccessGrantsLocation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccessGrantsLocationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccessGrantsLocation resources.
 */
export interface AccessGrantsLocationState {
    /**
     * Amazon Resource Name (ARN) of the S3 Access Grants location.
     */
    accessGrantsLocationArn?: pulumi.Input<string>;
    /**
     * Unique ID of the S3 Access Grants location.
     */
    accessGrantsLocationId?: pulumi.Input<string>;
    accountId?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role that S3 Access Grants should use when fulfilling runtime access
     * requests to the location.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * The default S3 URI `s3://` or the URI to a custom location, a specific bucket or prefix.
     */
    locationScope?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a AccessGrantsLocation resource.
 */
export interface AccessGrantsLocationArgs {
    accountId?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role that S3 Access Grants should use when fulfilling runtime access
     * requests to the location.
     */
    iamRoleArn: pulumi.Input<string>;
    /**
     * The default S3 URI `s3://` or the URI to a custom location, a specific bucket or prefix.
     */
    locationScope: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
