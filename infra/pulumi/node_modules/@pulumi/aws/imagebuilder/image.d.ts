import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Image Builder Image.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.imagebuilder.Image("example", {
 *     distributionConfigurationArn: exampleAwsImagebuilderDistributionConfiguration.arn,
 *     imageRecipeArn: exampleAwsImagebuilderImageRecipe.arn,
 *     infrastructureConfigurationArn: exampleAwsImagebuilderInfrastructureConfiguration.arn,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Image Builder image.
 *
 * Using `pulumi import`, import `aws_imagebuilder_image` resources using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_imagebuilder_image.example arn:aws:imagebuilder:us-east-1:123456789012:image/example/1.0.0/1
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
     * Amazon Resource Name (ARN) of the image.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the container recipe.
     */
    readonly containerRecipeArn: pulumi.Output<string | undefined>;
    /**
     * Date the image was created.
     */
    readonly dateCreated: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Distribution Configuration.
     */
    readonly distributionConfigurationArn: pulumi.Output<string | undefined>;
    /**
     * Whether additional information about the image being created is collected. Defaults to `true`.
     */
    readonly enhancedImageMetadataEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Amazon Resource Name (ARN) of the service-linked role to be used by Image Builder to [execute workflows](https://docs.aws.amazon.com/imagebuilder/latest/userguide/manage-image-workflows.html).
     */
    readonly executionRole: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the image recipe.
     */
    readonly imageRecipeArn: pulumi.Output<string | undefined>;
    /**
     * Configuration block with image scanning configuration. Detailed below.
     */
    readonly imageScanningConfiguration: pulumi.Output<outputs.imagebuilder.ImageImageScanningConfiguration>;
    /**
     * Configuration block with image tests configuration. Detailed below.
     */
    readonly imageTestsConfiguration: pulumi.Output<outputs.imagebuilder.ImageImageTestsConfiguration>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Infrastructure Configuration.
     *
     * The following arguments are optional:
     */
    readonly infrastructureConfigurationArn: pulumi.Output<string>;
    /**
     * Name of the AMI.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Operating System version of the image.
     */
    readonly osVersion: pulumi.Output<string>;
    /**
     * List of objects with resources created by the image.
     */
    readonly outputResources: pulumi.Output<outputs.imagebuilder.ImageOutputResource[]>;
    /**
     * Platform of the image.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags for the Image Builder Image. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Version of the image.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Configuration block with the workflow configuration. Detailed below.
     */
    readonly workflows: pulumi.Output<outputs.imagebuilder.ImageWorkflow[]>;
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
     * Amazon Resource Name (ARN) of the image.
     */
    arn?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the container recipe.
     */
    containerRecipeArn?: pulumi.Input<string>;
    /**
     * Date the image was created.
     */
    dateCreated?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Distribution Configuration.
     */
    distributionConfigurationArn?: pulumi.Input<string>;
    /**
     * Whether additional information about the image being created is collected. Defaults to `true`.
     */
    enhancedImageMetadataEnabled?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the service-linked role to be used by Image Builder to [execute workflows](https://docs.aws.amazon.com/imagebuilder/latest/userguide/manage-image-workflows.html).
     */
    executionRole?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the image recipe.
     */
    imageRecipeArn?: pulumi.Input<string>;
    /**
     * Configuration block with image scanning configuration. Detailed below.
     */
    imageScanningConfiguration?: pulumi.Input<inputs.imagebuilder.ImageImageScanningConfiguration>;
    /**
     * Configuration block with image tests configuration. Detailed below.
     */
    imageTestsConfiguration?: pulumi.Input<inputs.imagebuilder.ImageImageTestsConfiguration>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Infrastructure Configuration.
     *
     * The following arguments are optional:
     */
    infrastructureConfigurationArn?: pulumi.Input<string>;
    /**
     * Name of the AMI.
     */
    name?: pulumi.Input<string>;
    /**
     * Operating System version of the image.
     */
    osVersion?: pulumi.Input<string>;
    /**
     * List of objects with resources created by the image.
     */
    outputResources?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageOutputResource>[]>;
    /**
     * Platform of the image.
     */
    platform?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the Image Builder Image. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Version of the image.
     */
    version?: pulumi.Input<string>;
    /**
     * Configuration block with the workflow configuration. Detailed below.
     */
    workflows?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageWorkflow>[]>;
}
/**
 * The set of arguments for constructing a Image resource.
 */
export interface ImageArgs {
    /**
     * Amazon Resource Name (ARN) of the container recipe.
     */
    containerRecipeArn?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Distribution Configuration.
     */
    distributionConfigurationArn?: pulumi.Input<string>;
    /**
     * Whether additional information about the image being created is collected. Defaults to `true`.
     */
    enhancedImageMetadataEnabled?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the service-linked role to be used by Image Builder to [execute workflows](https://docs.aws.amazon.com/imagebuilder/latest/userguide/manage-image-workflows.html).
     */
    executionRole?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the image recipe.
     */
    imageRecipeArn?: pulumi.Input<string>;
    /**
     * Configuration block with image scanning configuration. Detailed below.
     */
    imageScanningConfiguration?: pulumi.Input<inputs.imagebuilder.ImageImageScanningConfiguration>;
    /**
     * Configuration block with image tests configuration. Detailed below.
     */
    imageTestsConfiguration?: pulumi.Input<inputs.imagebuilder.ImageImageTestsConfiguration>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Infrastructure Configuration.
     *
     * The following arguments are optional:
     */
    infrastructureConfigurationArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the Image Builder Image. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block with the workflow configuration. Detailed below.
     */
    workflows?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImageWorkflow>[]>;
}
