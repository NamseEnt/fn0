import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an [IAM service-linked role](https://docs.aws.amazon.com/IAM/latest/UserGuide/using-service-linked-roles.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const elasticbeanstalk = new aws.iam.ServiceLinkedRole("elasticbeanstalk", {awsServiceName: "elasticbeanstalk.amazonaws.com"});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the IAM service-linked role.
 *
 * Using `pulumi import`, import IAM service-linked roles using role ARN. For example:
 *
 * console
 *
 * % pulumi import aws_iam_service_linked_role.elasticbeanstalk arn:aws:iam::123456789012:role/aws-service-role/elasticbeanstalk.amazonaws.com/AWSServiceRoleForElasticBeanstalk
 */
export declare class ServiceLinkedRole extends pulumi.CustomResource {
    /**
     * Get an existing ServiceLinkedRole resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceLinkedRoleState, opts?: pulumi.CustomResourceOptions): ServiceLinkedRole;
    /**
     * Returns true if the given object is an instance of ServiceLinkedRole.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceLinkedRole;
    /**
     * The Amazon Resource Name (ARN) specifying the role.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS service to which this role is attached. You use a string similar to a URL but without the `http://` in front. For example: `elasticbeanstalk.amazonaws.com`. To find the full list of services that support service-linked roles, check [the docs](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_aws-services-that-work-with-iam.html).
     */
    readonly awsServiceName: pulumi.Output<string>;
    /**
     * The creation date of the IAM role.
     */
    readonly createDate: pulumi.Output<string>;
    /**
     * Additional string appended to the role name. Not all AWS services support custom suffixes.
     */
    readonly customSuffix: pulumi.Output<string | undefined>;
    /**
     * The description of the role.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the role.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The path of the role.
     */
    readonly path: pulumi.Output<string>;
    /**
     * Key-value mapping of tags for the IAM role. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The stable and unique string identifying the role.
     */
    readonly uniqueId: pulumi.Output<string>;
    /**
     * Create a ServiceLinkedRole resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceLinkedRoleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceLinkedRole resources.
 */
export interface ServiceLinkedRoleState {
    /**
     * The Amazon Resource Name (ARN) specifying the role.
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS service to which this role is attached. You use a string similar to a URL but without the `http://` in front. For example: `elasticbeanstalk.amazonaws.com`. To find the full list of services that support service-linked roles, check [the docs](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_aws-services-that-work-with-iam.html).
     */
    awsServiceName?: pulumi.Input<string>;
    /**
     * The creation date of the IAM role.
     */
    createDate?: pulumi.Input<string>;
    /**
     * Additional string appended to the role name. Not all AWS services support custom suffixes.
     */
    customSuffix?: pulumi.Input<string>;
    /**
     * The description of the role.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the role.
     */
    name?: pulumi.Input<string>;
    /**
     * The path of the role.
     */
    path?: pulumi.Input<string>;
    /**
     * Key-value mapping of tags for the IAM role. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The stable and unique string identifying the role.
     */
    uniqueId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServiceLinkedRole resource.
 */
export interface ServiceLinkedRoleArgs {
    /**
     * The AWS service to which this role is attached. You use a string similar to a URL but without the `http://` in front. For example: `elasticbeanstalk.amazonaws.com`. To find the full list of services that support service-linked roles, check [the docs](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_aws-services-that-work-with-iam.html).
     */
    awsServiceName: pulumi.Input<string>;
    /**
     * Additional string appended to the role name. Not all AWS services support custom suffixes.
     */
    customSuffix?: pulumi.Input<string>;
    /**
     * The description of the role.
     */
    description?: pulumi.Input<string>;
    /**
     * Key-value mapping of tags for the IAM role. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
