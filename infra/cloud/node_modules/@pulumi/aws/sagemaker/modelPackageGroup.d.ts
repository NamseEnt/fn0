import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a SageMaker AI Model Package Group resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.ModelPackageGroup("example", {modelPackageGroupName: "example"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Model Package Groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/modelPackageGroup:ModelPackageGroup test_model_package_group my-code-repo
 * ```
 */
export declare class ModelPackageGroup extends pulumi.CustomResource {
    /**
     * Get an existing ModelPackageGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ModelPackageGroupState, opts?: pulumi.CustomResourceOptions): ModelPackageGroup;
    /**
     * Returns true if the given object is an instance of ModelPackageGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ModelPackageGroup;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Model Package Group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description for the model group.
     */
    readonly modelPackageGroupDescription: pulumi.Output<string | undefined>;
    /**
     * The name of the model group.
     */
    readonly modelPackageGroupName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Create a ModelPackageGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ModelPackageGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ModelPackageGroup resources.
 */
export interface ModelPackageGroupState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Model Package Group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description for the model group.
     */
    modelPackageGroupDescription?: pulumi.Input<string>;
    /**
     * The name of the model group.
     */
    modelPackageGroupName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
 * The set of arguments for constructing a ModelPackageGroup resource.
 */
export interface ModelPackageGroupArgs {
    /**
     * A description for the model group.
     */
    modelPackageGroupDescription?: pulumi.Input<string>;
    /**
     * The name of the model group.
     */
    modelPackageGroupName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
