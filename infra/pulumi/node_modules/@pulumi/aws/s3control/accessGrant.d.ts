import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an S3 Access Grant.
 * Each access grant has its own ID and gives an IAM user or role or a directory user, or group (the grantee) access to a registered location. You determine the level of access, such as `READ` or `READWRITE`.
 * Before you can create a grant, you must have an S3 Access Grants instance in the same Region as the S3 data.
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
 *     locationScope: `s3://${exampleAwsS3Bucket.bucket}/prefixA*`,
 * }, {
 *     dependsOn: [example],
 * });
 * const exampleAccessGrant = new aws.s3control.AccessGrant("example", {
 *     accessGrantsLocationId: exampleAccessGrantsLocation.accessGrantsLocationId,
 *     permission: "READ",
 *     accessGrantsLocationConfiguration: {
 *         s3SubPrefix: "prefixB*",
 *     },
 *     grantee: {
 *         granteeType: "IAM",
 *         granteeIdentifier: exampleAwsIamUser.arn,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Access Grants using the `account_id` and `access_grant_id`, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/accessGrant:AccessGrant example 123456789012,04549c5e-2f3c-4a07-824d-2cafe720aa22
 * ```
 */
export declare class AccessGrant extends pulumi.CustomResource {
    /**
     * Get an existing AccessGrant resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccessGrantState, opts?: pulumi.CustomResourceOptions): AccessGrant;
    /**
     * Returns true if the given object is an instance of AccessGrant.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccessGrant;
    /**
     * Amazon Resource Name (ARN) of the S3 Access Grant.
     */
    readonly accessGrantArn: pulumi.Output<string>;
    /**
     * Unique ID of the S3 Access Grant.
     */
    readonly accessGrantId: pulumi.Output<string>;
    /**
     * See Location Configuration below for more details.
     */
    readonly accessGrantsLocationConfiguration: pulumi.Output<outputs.s3control.AccessGrantAccessGrantsLocationConfiguration | undefined>;
    /**
     * The ID of the S3 Access Grants location to with the access grant is giving access.
     */
    readonly accessGrantsLocationId: pulumi.Output<string>;
    readonly accountId: pulumi.Output<string>;
    /**
     * The access grant's scope.
     */
    readonly grantScope: pulumi.Output<string>;
    /**
     * See Grantee below for more details.
     */
    readonly grantee: pulumi.Output<outputs.s3control.AccessGrantGrantee | undefined>;
    /**
     * The access grant's level of access. Valid values: `READ`, `WRITE`, `READWRITE`.
     */
    readonly permission: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * If you are creating an access grant that grants access to only one object, set this to `Object`. Valid values: `Object`.
     */
    readonly s3PrefixType: pulumi.Output<string | undefined>;
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
     * Create a AccessGrant resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccessGrantArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccessGrant resources.
 */
export interface AccessGrantState {
    /**
     * Amazon Resource Name (ARN) of the S3 Access Grant.
     */
    accessGrantArn?: pulumi.Input<string>;
    /**
     * Unique ID of the S3 Access Grant.
     */
    accessGrantId?: pulumi.Input<string>;
    /**
     * See Location Configuration below for more details.
     */
    accessGrantsLocationConfiguration?: pulumi.Input<inputs.s3control.AccessGrantAccessGrantsLocationConfiguration>;
    /**
     * The ID of the S3 Access Grants location to with the access grant is giving access.
     */
    accessGrantsLocationId?: pulumi.Input<string>;
    accountId?: pulumi.Input<string>;
    /**
     * The access grant's scope.
     */
    grantScope?: pulumi.Input<string>;
    /**
     * See Grantee below for more details.
     */
    grantee?: pulumi.Input<inputs.s3control.AccessGrantGrantee>;
    /**
     * The access grant's level of access. Valid values: `READ`, `WRITE`, `READWRITE`.
     */
    permission?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * If you are creating an access grant that grants access to only one object, set this to `Object`. Valid values: `Object`.
     */
    s3PrefixType?: pulumi.Input<string>;
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
 * The set of arguments for constructing a AccessGrant resource.
 */
export interface AccessGrantArgs {
    /**
     * See Location Configuration below for more details.
     */
    accessGrantsLocationConfiguration?: pulumi.Input<inputs.s3control.AccessGrantAccessGrantsLocationConfiguration>;
    /**
     * The ID of the S3 Access Grants location to with the access grant is giving access.
     */
    accessGrantsLocationId: pulumi.Input<string>;
    accountId?: pulumi.Input<string>;
    /**
     * See Grantee below for more details.
     */
    grantee?: pulumi.Input<inputs.s3control.AccessGrantGrantee>;
    /**
     * The access grant's level of access. Valid values: `READ`, `WRITE`, `READWRITE`.
     */
    permission: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * If you are creating an access grant that grants access to only one object, set this to `Object`. Valid values: `Object`.
     */
    s3PrefixType?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
