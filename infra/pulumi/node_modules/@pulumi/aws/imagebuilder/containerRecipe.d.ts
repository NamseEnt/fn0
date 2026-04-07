import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Image Builder Container Recipe.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.imagebuilder.ContainerRecipe("example", {
 *     name: "example",
 *     version: "1.0.0",
 *     containerType: "DOCKER",
 *     parentImage: "arn:aws:imagebuilder:eu-central-1:aws:image/amazon-linux-x86-latest/x.x.x",
 *     targetRepository: {
 *         repositoryName: exampleAwsEcrRepository.name,
 *         service: "ECR",
 *     },
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
 *     dockerfileTemplateData: `FROM {{{ imagebuilder:parentImage }}}
 * {{{ imagebuilder:environments }}}
 * {{{ imagebuilder:components }}}
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Image Builder container recipe.
 *
 * Using `pulumi import`, import `aws_imagebuilder_container_recipe` resources using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_imagebuilder_container_recipe.example arn:aws:imagebuilder:us-east-1:123456789012:container-recipe/example/1.0.0
 */
export declare class ContainerRecipe extends pulumi.CustomResource {
    /**
     * Get an existing ContainerRecipe resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContainerRecipeState, opts?: pulumi.CustomResourceOptions): ContainerRecipe;
    /**
     * Returns true if the given object is an instance of ContainerRecipe.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContainerRecipe;
    /**
     * (Required) Amazon Resource Name (ARN) of the container recipe.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Ordered configuration block(s) with components for the container recipe. Detailed below.
     */
    readonly components: pulumi.Output<outputs.imagebuilder.ContainerRecipeComponent[]>;
    /**
     * The type of the container to create. Valid values: `DOCKER`.
     */
    readonly containerType: pulumi.Output<string>;
    /**
     * Date the container recipe was created.
     */
    readonly dateCreated: pulumi.Output<string>;
    /**
     * The description of the container recipe.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The Dockerfile template used to build the image as an inline data blob.
     */
    readonly dockerfileTemplateData: pulumi.Output<string>;
    /**
     * The Amazon S3 URI for the Dockerfile that will be used to build the container image.
     */
    readonly dockerfileTemplateUri: pulumi.Output<string | undefined>;
    /**
     * A flag that indicates if the target container is encrypted.
     */
    readonly encrypted: pulumi.Output<boolean>;
    /**
     * Configuration block used to configure an instance for building and testing container images. Detailed below.
     */
    readonly instanceConfiguration: pulumi.Output<outputs.imagebuilder.ContainerRecipeInstanceConfiguration | undefined>;
    /**
     * The KMS key used to encrypt the container image.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * The name of the container recipe.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Owner of the container recipe.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * The base image for the container recipe.
     */
    readonly parentImage: pulumi.Output<string>;
    /**
     * Platform of the container recipe.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Specifies the operating system platform when you use a custom base image.
     */
    readonly platformOverride: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags for the container recipe. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The destination repository for the container image. Detailed below.
     */
    readonly targetRepository: pulumi.Output<outputs.imagebuilder.ContainerRecipeTargetRepository>;
    /**
     * Version of the container recipe.
     *
     * The following arguments are optional:
     */
    readonly version: pulumi.Output<string>;
    /**
     * The working directory to be used during build and test workflows.
     */
    readonly workingDirectory: pulumi.Output<string | undefined>;
    /**
     * Create a ContainerRecipe resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContainerRecipeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContainerRecipe resources.
 */
export interface ContainerRecipeState {
    /**
     * (Required) Amazon Resource Name (ARN) of the container recipe.
     */
    arn?: pulumi.Input<string>;
    /**
     * Ordered configuration block(s) with components for the container recipe. Detailed below.
     */
    components?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ContainerRecipeComponent>[]>;
    /**
     * The type of the container to create. Valid values: `DOCKER`.
     */
    containerType?: pulumi.Input<string>;
    /**
     * Date the container recipe was created.
     */
    dateCreated?: pulumi.Input<string>;
    /**
     * The description of the container recipe.
     */
    description?: pulumi.Input<string>;
    /**
     * The Dockerfile template used to build the image as an inline data blob.
     */
    dockerfileTemplateData?: pulumi.Input<string>;
    /**
     * The Amazon S3 URI for the Dockerfile that will be used to build the container image.
     */
    dockerfileTemplateUri?: pulumi.Input<string>;
    /**
     * A flag that indicates if the target container is encrypted.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * Configuration block used to configure an instance for building and testing container images. Detailed below.
     */
    instanceConfiguration?: pulumi.Input<inputs.imagebuilder.ContainerRecipeInstanceConfiguration>;
    /**
     * The KMS key used to encrypt the container image.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of the container recipe.
     */
    name?: pulumi.Input<string>;
    /**
     * Owner of the container recipe.
     */
    owner?: pulumi.Input<string>;
    /**
     * The base image for the container recipe.
     */
    parentImage?: pulumi.Input<string>;
    /**
     * Platform of the container recipe.
     */
    platform?: pulumi.Input<string>;
    /**
     * Specifies the operating system platform when you use a custom base image.
     */
    platformOverride?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the container recipe. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The destination repository for the container image. Detailed below.
     */
    targetRepository?: pulumi.Input<inputs.imagebuilder.ContainerRecipeTargetRepository>;
    /**
     * Version of the container recipe.
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
 * The set of arguments for constructing a ContainerRecipe resource.
 */
export interface ContainerRecipeArgs {
    /**
     * Ordered configuration block(s) with components for the container recipe. Detailed below.
     */
    components: pulumi.Input<pulumi.Input<inputs.imagebuilder.ContainerRecipeComponent>[]>;
    /**
     * The type of the container to create. Valid values: `DOCKER`.
     */
    containerType: pulumi.Input<string>;
    /**
     * The description of the container recipe.
     */
    description?: pulumi.Input<string>;
    /**
     * The Dockerfile template used to build the image as an inline data blob.
     */
    dockerfileTemplateData?: pulumi.Input<string>;
    /**
     * The Amazon S3 URI for the Dockerfile that will be used to build the container image.
     */
    dockerfileTemplateUri?: pulumi.Input<string>;
    /**
     * Configuration block used to configure an instance for building and testing container images. Detailed below.
     */
    instanceConfiguration?: pulumi.Input<inputs.imagebuilder.ContainerRecipeInstanceConfiguration>;
    /**
     * The KMS key used to encrypt the container image.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of the container recipe.
     */
    name?: pulumi.Input<string>;
    /**
     * The base image for the container recipe.
     */
    parentImage: pulumi.Input<string>;
    /**
     * Specifies the operating system platform when you use a custom base image.
     */
    platformOverride?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the container recipe. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The destination repository for the container image. Detailed below.
     */
    targetRepository: pulumi.Input<inputs.imagebuilder.ContainerRecipeTargetRepository>;
    /**
     * Version of the container recipe.
     *
     * The following arguments are optional:
     */
    version: pulumi.Input<string>;
    /**
     * The working directory to be used during build and test workflows.
     */
    workingDirectory?: pulumi.Input<string>;
}
