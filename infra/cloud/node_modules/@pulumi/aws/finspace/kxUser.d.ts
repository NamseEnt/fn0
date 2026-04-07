import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS FinSpace Kx User.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "Example KMS Key",
 *     deletionWindowInDays: 7,
 * });
 * const exampleKxEnvironment = new aws.finspace.KxEnvironment("example", {
 *     name: "my-tf-kx-environment",
 *     kmsKeyId: example.arn,
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example-role",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "ec2.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const exampleKxUser = new aws.finspace.KxUser("example", {
 *     name: "my-tf-kx-user",
 *     environmentId: exampleKxEnvironment.id,
 *     iamRole: exampleRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an AWS FinSpace Kx User using the `id` (environment ID and user name, comma-delimited). For example:
 *
 * ```sh
 * $ pulumi import aws:finspace/kxUser:KxUser example n3ceo7wqxoxcti5tujqwzs,my-tf-kx-user
 * ```
 */
export declare class KxUser extends pulumi.CustomResource {
    /**
     * Get an existing KxUser resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KxUserState, opts?: pulumi.CustomResourceOptions): KxUser;
    /**
     * Returns true if the given object is an instance of KxUser.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KxUser;
    /**
     * Amazon Resource Name (ARN) identifier of the KX user.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Unique identifier for the KX environment.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * IAM role ARN to be associated with the user.
     *
     * The following arguments are optional:
     */
    readonly iamRole: pulumi.Output<string>;
    /**
     * A unique identifier for the user.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a KxUser resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KxUserArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KxUser resources.
 */
export interface KxUserState {
    /**
     * Amazon Resource Name (ARN) identifier of the KX user.
     */
    arn?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * IAM role ARN to be associated with the user.
     *
     * The following arguments are optional:
     */
    iamRole?: pulumi.Input<string>;
    /**
     * A unique identifier for the user.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a KxUser resource.
 */
export interface KxUserArgs {
    /**
     * Unique identifier for the KX environment.
     */
    environmentId: pulumi.Input<string>;
    /**
     * IAM role ARN to be associated with the user.
     *
     * The following arguments are optional:
     */
    iamRole: pulumi.Input<string>;
    /**
     * A unique identifier for the user.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
