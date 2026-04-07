import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a SageMaker AI Image Version resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.ImageVersion("example", {
 *     imageName: test.id,
 *     baseImage: "012345678912.dkr.ecr.us-west-2.amazonaws.com/image:latest",
 * });
 * ```
 *
 * ### With Aliases
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.sagemaker.ImageVersion("test", {
 *     imageName: testAwsSagemakerImage.id,
 *     baseImage: "012345678912.dkr.ecr.us-west-2.amazonaws.com/image:latest",
 *     aliases: [
 *         "latest",
 *         "stable",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Image Versions using a comma-delimited string concatenating `image_name` and `version`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/imageVersion:ImageVersion example example-name,1
 * ```
 */
export declare class ImageVersion extends pulumi.CustomResource {
    /**
     * Get an existing ImageVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ImageVersionState, opts?: pulumi.CustomResourceOptions): ImageVersion;
    /**
     * Returns true if the given object is an instance of ImageVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ImageVersion;
    /**
     * A list of aliases for the image version.
     */
    readonly aliases: pulumi.Output<string[] | undefined>;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Image Version.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The registry path of the container image on which this image version is based.
     */
    readonly baseImage: pulumi.Output<string>;
    /**
     * The registry path of the container image that contains this image version.
     */
    readonly containerImage: pulumi.Output<string>;
    /**
     * Indicates Horovod compatibility.
     */
    readonly horovod: pulumi.Output<boolean | undefined>;
    readonly imageArn: pulumi.Output<string>;
    /**
     * The name of the image. Must be unique to your account.
     */
    readonly imageName: pulumi.Output<string>;
    /**
     * Indicates SageMaker AI job type compatibility. Valid values are: `TRAINING`, `INFERENCE`, and `NOTEBOOK_KERNEL`.
     */
    readonly jobType: pulumi.Output<string | undefined>;
    /**
     * The machine learning framework vended in the image version.
     */
    readonly mlFramework: pulumi.Output<string | undefined>;
    /**
     * Indicates CPU or GPU compatibility. Valid values are: `CPU` and `GPU`.
     */
    readonly processor: pulumi.Output<string | undefined>;
    /**
     * The supported programming language and its version.
     */
    readonly programmingLang: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The maintainer description of the image version.
     */
    readonly releaseNotes: pulumi.Output<string | undefined>;
    /**
     * The stability of the image version, specified by the maintainer. Valid values are: `NOT_PROVIDED`, `STABLE`, `TO_BE_ARCHIVED`, and `ARCHIVED`.
     */
    readonly vendorGuidance: pulumi.Output<string | undefined>;
    /**
     * The version of the image. If not specified, the latest version is described.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a ImageVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ImageVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ImageVersion resources.
 */
export interface ImageVersionState {
    /**
     * A list of aliases for the image version.
     */
    aliases?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Image Version.
     */
    arn?: pulumi.Input<string>;
    /**
     * The registry path of the container image on which this image version is based.
     */
    baseImage?: pulumi.Input<string>;
    /**
     * The registry path of the container image that contains this image version.
     */
    containerImage?: pulumi.Input<string>;
    /**
     * Indicates Horovod compatibility.
     */
    horovod?: pulumi.Input<boolean>;
    imageArn?: pulumi.Input<string>;
    /**
     * The name of the image. Must be unique to your account.
     */
    imageName?: pulumi.Input<string>;
    /**
     * Indicates SageMaker AI job type compatibility. Valid values are: `TRAINING`, `INFERENCE`, and `NOTEBOOK_KERNEL`.
     */
    jobType?: pulumi.Input<string>;
    /**
     * The machine learning framework vended in the image version.
     */
    mlFramework?: pulumi.Input<string>;
    /**
     * Indicates CPU or GPU compatibility. Valid values are: `CPU` and `GPU`.
     */
    processor?: pulumi.Input<string>;
    /**
     * The supported programming language and its version.
     */
    programmingLang?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The maintainer description of the image version.
     */
    releaseNotes?: pulumi.Input<string>;
    /**
     * The stability of the image version, specified by the maintainer. Valid values are: `NOT_PROVIDED`, `STABLE`, `TO_BE_ARCHIVED`, and `ARCHIVED`.
     */
    vendorGuidance?: pulumi.Input<string>;
    /**
     * The version of the image. If not specified, the latest version is described.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a ImageVersion resource.
 */
export interface ImageVersionArgs {
    /**
     * A list of aliases for the image version.
     */
    aliases?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The registry path of the container image on which this image version is based.
     */
    baseImage: pulumi.Input<string>;
    /**
     * Indicates Horovod compatibility.
     */
    horovod?: pulumi.Input<boolean>;
    /**
     * The name of the image. Must be unique to your account.
     */
    imageName: pulumi.Input<string>;
    /**
     * Indicates SageMaker AI job type compatibility. Valid values are: `TRAINING`, `INFERENCE`, and `NOTEBOOK_KERNEL`.
     */
    jobType?: pulumi.Input<string>;
    /**
     * The machine learning framework vended in the image version.
     */
    mlFramework?: pulumi.Input<string>;
    /**
     * Indicates CPU or GPU compatibility. Valid values are: `CPU` and `GPU`.
     */
    processor?: pulumi.Input<string>;
    /**
     * The supported programming language and its version.
     */
    programmingLang?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The maintainer description of the image version.
     */
    releaseNotes?: pulumi.Input<string>;
    /**
     * The stability of the image version, specified by the maintainer. Valid values are: `NOT_PROVIDED`, `STABLE`, `TO_BE_ARCHIVED`, and `ARCHIVED`.
     */
    vendorGuidance?: pulumi.Input<string>;
}
