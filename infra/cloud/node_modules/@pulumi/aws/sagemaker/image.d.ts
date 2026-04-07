import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a SageMaker AI Image resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Image("example", {
 *     imageName: "example",
 *     roleArn: test.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Code Images using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/image:Image test_image my-code-repo
 * ```
 */
export declare class Image extends pulumi.CustomResource {
    /**
     * Get an existing Image resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ImageState, opts?: pulumi.CustomResourceOptions): Image;
    /**
     * Returns true if the given object is an instance of Image.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Image;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Image.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the image.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The display name of the image. When the image is added to a domain (must be unique to the domain).
     */
    readonly displayName: pulumi.Output<string | undefined>;
    /**
     * The name of the image. Must be unique to your account.
     */
    readonly imageName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that enables Amazon SageMaker AI to perform tasks on your behalf.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Image resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ImageArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Image resources.
 */
export interface ImageState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Image.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the image.
     */
    description?: pulumi.Input<string>;
    /**
     * The display name of the image. When the image is added to a domain (must be unique to the domain).
     */
    displayName?: pulumi.Input<string>;
    /**
     * The name of the image. Must be unique to your account.
     */
    imageName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that enables Amazon SageMaker AI to perform tasks on your behalf.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Image resource.
 */
export interface ImageArgs {
    /**
     * The description of the image.
     */
    description?: pulumi.Input<string>;
    /**
     * The display name of the image. When the image is added to a domain (must be unique to the domain).
     */
    displayName?: pulumi.Input<string>;
    /**
     * The name of the image. Must be unique to your account.
     */
    imageName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that enables Amazon SageMaker AI to perform tasks on your behalf.
     */
    roleArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
