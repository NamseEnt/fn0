import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Image Builder Image Pipeline.
 *
 * > **NOTE:** Starting with version `5.74.0`, lifecycle meta-argument `replaceTriggeredBy` must be used in order to prevent a dependency error on destroy.
 */
export declare class ImagePipeline extends pulumi.CustomResource {
    /**
     * Get an existing ImagePipeline resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ImagePipelineState, opts?: pulumi.CustomResourceOptions): ImagePipeline;
    /**
     * Returns true if the given object is an instance of ImagePipeline.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ImagePipeline;
    /**
     * Amazon Resource Name (ARN) of the image pipeline.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the container recipe.
     */
    readonly containerRecipeArn: pulumi.Output<string | undefined>;
    /**
     * Date the image pipeline was created.
     */
    readonly dateCreated: pulumi.Output<string>;
    /**
     * Date the image pipeline was last run.
     */
    readonly dateLastRun: pulumi.Output<string>;
    /**
     * Date the image pipeline will run next.
     */
    readonly dateNextRun: pulumi.Output<string>;
    /**
     * Date the image pipeline was updated.
     */
    readonly dateUpdated: pulumi.Output<string>;
    /**
     * Description of the image pipeline.
     */
    readonly description: pulumi.Output<string | undefined>;
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
    readonly executionRole: pulumi.Output<string | undefined>;
    /**
     * Amazon Resource Name (ARN) of the image recipe.
     */
    readonly imageRecipeArn: pulumi.Output<string | undefined>;
    /**
     * Configuration block with image scanning configuration. Detailed below.
     */
    readonly imageScanningConfiguration: pulumi.Output<outputs.imagebuilder.ImagePipelineImageScanningConfiguration>;
    /**
     * Configuration block with image tests configuration. Detailed below.
     */
    readonly imageTestsConfiguration: pulumi.Output<outputs.imagebuilder.ImagePipelineImageTestsConfiguration>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Infrastructure Configuration.
     */
    readonly infrastructureConfigurationArn: pulumi.Output<string>;
    /**
     * Name of the image pipeline.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Platform of the image pipeline.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block with schedule settings. Detailed below.
     */
    readonly schedule: pulumi.Output<outputs.imagebuilder.ImagePipelineSchedule | undefined>;
    /**
     * Status of the image pipeline. Valid values are `DISABLED` and `ENABLED`. Defaults to `ENABLED`.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags for the image pipeline. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Configuration block with the workflow configuration. Detailed below.
     */
    readonly workflows: pulumi.Output<outputs.imagebuilder.ImagePipelineWorkflow[]>;
    /**
     * Create a ImagePipeline resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ImagePipelineArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ImagePipeline resources.
 */
export interface ImagePipelineState {
    /**
     * Amazon Resource Name (ARN) of the image pipeline.
     */
    arn?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the container recipe.
     */
    containerRecipeArn?: pulumi.Input<string>;
    /**
     * Date the image pipeline was created.
     */
    dateCreated?: pulumi.Input<string>;
    /**
     * Date the image pipeline was last run.
     */
    dateLastRun?: pulumi.Input<string>;
    /**
     * Date the image pipeline will run next.
     */
    dateNextRun?: pulumi.Input<string>;
    /**
     * Date the image pipeline was updated.
     */
    dateUpdated?: pulumi.Input<string>;
    /**
     * Description of the image pipeline.
     */
    description?: pulumi.Input<string>;
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
    imageScanningConfiguration?: pulumi.Input<inputs.imagebuilder.ImagePipelineImageScanningConfiguration>;
    /**
     * Configuration block with image tests configuration. Detailed below.
     */
    imageTestsConfiguration?: pulumi.Input<inputs.imagebuilder.ImagePipelineImageTestsConfiguration>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Infrastructure Configuration.
     */
    infrastructureConfigurationArn?: pulumi.Input<string>;
    /**
     * Name of the image pipeline.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Platform of the image pipeline.
     */
    platform?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block with schedule settings. Detailed below.
     */
    schedule?: pulumi.Input<inputs.imagebuilder.ImagePipelineSchedule>;
    /**
     * Status of the image pipeline. Valid values are `DISABLED` and `ENABLED`. Defaults to `ENABLED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the image pipeline. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Configuration block with the workflow configuration. Detailed below.
     */
    workflows?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImagePipelineWorkflow>[]>;
}
/**
 * The set of arguments for constructing a ImagePipeline resource.
 */
export interface ImagePipelineArgs {
    /**
     * Amazon Resource Name (ARN) of the container recipe.
     */
    containerRecipeArn?: pulumi.Input<string>;
    /**
     * Description of the image pipeline.
     */
    description?: pulumi.Input<string>;
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
    imageScanningConfiguration?: pulumi.Input<inputs.imagebuilder.ImagePipelineImageScanningConfiguration>;
    /**
     * Configuration block with image tests configuration. Detailed below.
     */
    imageTestsConfiguration?: pulumi.Input<inputs.imagebuilder.ImagePipelineImageTestsConfiguration>;
    /**
     * Amazon Resource Name (ARN) of the Image Builder Infrastructure Configuration.
     */
    infrastructureConfigurationArn: pulumi.Input<string>;
    /**
     * Name of the image pipeline.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block with schedule settings. Detailed below.
     */
    schedule?: pulumi.Input<inputs.imagebuilder.ImagePipelineSchedule>;
    /**
     * Status of the image pipeline. Valid values are `DISABLED` and `ENABLED`. Defaults to `ENABLED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the image pipeline. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block with the workflow configuration. Detailed below.
     */
    workflows?: pulumi.Input<pulumi.Input<inputs.imagebuilder.ImagePipelineWorkflow>[]>;
}
