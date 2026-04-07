import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Image Builder Image Recipe.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.imagebuilder.ImageRecipe("example", {
 *     blockDeviceMappings: [{
 *         deviceName: "/dev/xvdb",
 *         ebs: {
 *             deleteOnTermination: "true",
 *             volumeSize: 100,
 *             volumeType: "gp2",
 *         },
 *     }],
 *     components: [{
 *         componentArn: exampleAwsImagebuilderComponent.arn,
 *         parameters: [
 *             {
 *                 name: "Parameter1",
 *                 value: "Value1",
 *             },
 *             {
 *                 name: "Parameter2",
 *                 value: "Value2",
 *             },
 *         ],
 *     }],
 *     name: "example",
 *     parentImage: `arn:${current.partition}:imagebuilder:${currentAwsRegion.region}:aws:image/amazon-linux-2-x86/x.x.x`,
 *     version: "1.0.0",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Image Builder image recipe.
 *
 * Using `pulumi import`, import `aws_imagebuilder_image_recipe` resources using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_imagebuilder_image_recipe.example arn:aws:imagebuilder:us-east-1:123456789012:image-recipe/example/1.0.0
 */
export declare class ImageRecipe extends pulumi.CustomResource {
    /**
     * Get an existing ImageRecipe resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ImageRecipeState, opts?: pulumi.CustomResourceOptions): ImageRecipe;
    /**
     * Returns true if the given object is an instance of ImageRecipe.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ImageRecipe;
    /**
     * Tags that are applied to the AMI that Image Builder creates during the Build phase prior to image distribution. Maximum of 50 tags.
     */
    readonly amiTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Amazon Resource Name (ARN) of the image recipe.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block(s) with block device mappings for the image recipe. Detailed below.
     */
    readonly blockDeviceMappings: pulumi.Output<outputs.imagebuilder.ImageRecipeBlockDeviceMapping[] | undefined>;
    /**
     * Ordered configuration block(s) with components for the image recipe. Detailed below.
     */
    readonly components: pulumi.Output<outputs.imagebuilder.ImageRecipeComponent[]>;
    /**
     * Date the image recipe was created.
     */
    readonly dateCreated: pulumi.Output<string>;
    /**
     * Description of the image recipe.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the image recipe.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Owner of the image recipe.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * The image recipe uses this image as a base from which to build your customized image. The value can be the base image ARN, an AMI ID, or an SSM Parameter referencing the AMI. For an SSM Parameter, enter the prefix `ssm:`, followed by the parameter name or ARN.
     */
    readonly parentImage: pulumi.Output<string>;
    /**
     * Platform of the image recipe.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for the Systems Manager Agent installed by default by Image Builder. Detailed below.
     */
    readonly systemsManagerAgent: pulumi.Output<outputs.imagebuilder.ImageRecipeSystemsManagerAgent>;
    /**
     * Key-value map of resource tags for the image recipe. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Base64 encoded user data. Use this to provide commands or a command script to run when you launch your build instance.
     */
    readonly userDataBase64: pulumi.Output<string>;
    /**
     * The semantic version of the image recipe, which specifies the version in the following format, with numeric values in each position to indicate a specific version: major.minor.patch. For example: 1.0.0.
     *
     * The following arguments are optional:
     */
    readonly version: pulumi.Output<string>;
    /**
     * The working directory to be used during build and test workflows.
     */
    readonly workingDirectory: pulumi.Output<string | undefined>;
    /**
     * Create a ImageRecipe resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ImageRecipeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ImageRecipe resources.
 */
export interface ImageRecipeState {
    /**
     * Tags that are applied to the AMI that Image Builder creates during the Build phase prior to image distribution. Maximum of 50 tags.
     */
    amiTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Amazon Resource Name (ARN) of the image recipe.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block(s) with block device mappings for the image recipe. Detailed below.
     */
    blockDeviceMappings?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageRecipeBlockDeviceMapping>[]>;
    /**
     * Ordered configuration block(s) with components for the image recipe. Detailed below.
     */
    components?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageRecipeComponent>[]>;
    /**
     * Date the image recipe was created.
     */
    dateCreated?: pulumi.Input<string>;
    /**
     * Description of the image recipe.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the image recipe.
     */
    name?: pulumi.Input<string>;
    /**
     * Owner of the image recipe.
     */
    owner?: pulumi.Input<string>;
    /**
     * The image recipe uses this image as a base from which to build your customized image. The value can be the base image ARN, an AMI ID, or an SSM Parameter referencing the AMI. For an SSM Parameter, enter the prefix `ssm:`, followed by the parameter name or ARN.
     */
    parentImage?: pulumi.Input<string>;
    /**
     * Platform of the image recipe.
     */
    platform?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the Systems Manager Agent installed by default by Image Builder. Detailed below.
     */
    systemsManagerAgent?: pulumi.Input<inputs.imagebuilder.ImageRecipeSystemsManagerAgent>;
    /**
     * Key-value map of resource tags for the image recipe. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Base64 encoded user data. Use this to provide commands or a command script to run when you launch your build instance.
     */
    userDataBase64?: pulumi.Input<string>;
    /**
     * The semantic version of the image recipe, which specifies the version in the following format, with numeric values in each position to indicate a specific version: major.minor.patch. For example: 1.0.0.
     *
     * The following arguments are optional:
     */
    version?: pulumi.Input<string>;
    /**
     * The working directory to be used during build and test workflows.
     */
    workingDirectory?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ImageRecipe resource.
 */
export interface ImageRecipeArgs {
    /**
     * Tags that are applied to the AMI that Image Builder creates during the Build phase prior to image distribution. Maximum of 50 tags.
     */
    amiTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block(s) with block device mappings for the image recipe. Detailed below.
     */
    blockDeviceMappings?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageRecipeBlockDeviceMapping>[]>;
    /**
     * Ordered configuration block(s) with components for the image recipe. Detailed below.
     */
    components: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageRecipeComponent>[]>;
    /**
     * Description of the image recipe.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the image recipe.
     */
    name?: pulumi.Input<string>;
    /**
     * The image recipe uses this image as a base from which to build your customized image. The value can be the base image ARN, an AMI ID, or an SSM Parameter referencing the AMI. For an SSM Parameter, enter the prefix `ssm:`, followed by the parameter name or ARN.
     */
    parentImage: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the Systems Manager Agent installed by default by Image Builder. Detailed below.
     */
    systemsManagerAgent?: pulumi.Input<inputs.imagebuilder.ImageRecipeSystemsManagerAgent>;
    /**
     * Key-value map of resource tags for the image recipe. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Base64 encoded user data. Use this to provide commands or a command script to run when you launch your build instance.
     */
    userDataBase64?: pulumi.Input<string>;
    /**
     * The semantic version of the image recipe, which specifies the version in the following format, with numeric values in each position to indicate a specific version: major.minor.patch. For example: 1.0.0.
     *
     * The following arguments are optional:
     */
    version: pulumi.Input<string>;
    /**
     * The working directory to be used during build and test workflows.
     */
    workingDirectory?: pulumi.Input<string>;
}
