import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Comprehend Document Classifier.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const documents = new aws.s3.BucketObjectv2("documents", {});
 * const example = new aws.comprehend.DocumentClassifier("example", {
 *     name: "example",
 *     dataAccessRoleArn: exampleAwsIamRole.arn,
 *     languageCode: "en",
 *     inputDataConfig: {
 *         s3Uri: pulumi.interpolate`s3://${test.bucket}/${documents.key}`,
 *     },
 * }, {
 *     dependsOn: [exampleAwsIamRolePolicy],
 * });
 * const entities = new aws.s3.BucketObjectv2("entities", {});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Comprehend document classifier.
 *
 * Using `pulumi import`, import Comprehend Document Classifier using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_comprehend_document_classifier.example arn:aws:comprehend:us-west-2:123456789012:document_classifier/example
 */
export declare class DocumentClassifier extends pulumi.CustomResource {
    /**
     * Get an existing DocumentClassifier resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DocumentClassifierState, opts?: pulumi.CustomResourceOptions): DocumentClassifier;
    /**
     * Returns true if the given object is an instance of DocumentClassifier.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DocumentClassifier;
    /**
     * ARN of the Document Classifier version.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN for an IAM Role which allows Comprehend to read the training and testing data.
     */
    readonly dataAccessRoleArn: pulumi.Output<string>;
    /**
     * Configuration for the training and testing data.
     * See the `inputDataConfig` Configuration Block section below.
     */
    readonly inputDataConfig: pulumi.Output<outputs.comprehend.DocumentClassifierInputDataConfig>;
    /**
     * Two-letter language code for the language.
     * One of `en`, `es`, `fr`, `it`, `de`, or `pt`.
     */
    readonly languageCode: pulumi.Output<string>;
    /**
     * The document classification mode.
     * One of `MULTI_CLASS` or `MULTI_LABEL`.
     * `MULTI_CLASS` is also known as "Single Label" in the AWS Console.
     */
    readonly mode: pulumi.Output<string | undefined>;
    /**
     * KMS Key used to encrypt trained Document Classifiers.
     * Can be a KMS Key ID or a KMS Key ARN.
     */
    readonly modelKmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Name for the Document Classifier.
     * Has a maximum length of 63 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Configuration for the output results of training.
     * See the `outputDataConfig` Configuration Block section below.
     */
    readonly outputDataConfig: pulumi.Output<outputs.comprehend.DocumentClassifierOutputDataConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` Configuration Block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Name for the version of the Document Classifier.
     * Each version must have a unique name within the Document Classifier.
     * If omitted, the provider will assign a random, unique version name.
     * If explicitly set to `""`, no version name will be set.
     * Has a maximum length of 63 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     * Conflicts with `versionNamePrefix`.
     */
    readonly versionName: pulumi.Output<string>;
    /**
     * Creates a unique version name beginning with the specified prefix.
     * Has a maximum length of 37 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     * Conflicts with `versionName`.
     */
    readonly versionNamePrefix: pulumi.Output<string>;
    /**
     * KMS Key used to encrypt storage volumes during job processing.
     * Can be a KMS Key ID or a KMS Key ARN.
     */
    readonly volumeKmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Configuration parameters for VPC to contain Document Classifier resources.
     * See the `vpcConfig` Configuration Block section below.
     */
    readonly vpcConfig: pulumi.Output<outputs.comprehend.DocumentClassifierVpcConfig | undefined>;
    /**
     * Create a DocumentClassifier resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DocumentClassifierArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DocumentClassifier resources.
 */
export interface DocumentClassifierState {
    /**
     * ARN of the Document Classifier version.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN for an IAM Role which allows Comprehend to read the training and testing data.
     */
    dataAccessRoleArn?: pulumi.Input<string>;
    /**
     * Configuration for the training and testing data.
     * See the `inputDataConfig` Configuration Block section below.
     */
    inputDataConfig?: pulumi.Input<inputs.comprehend.DocumentClassifierInputDataConfig>;
    /**
     * Two-letter language code for the language.
     * One of `en`, `es`, `fr`, `it`, `de`, or `pt`.
     */
    languageCode?: pulumi.Input<string>;
    /**
     * The document classification mode.
     * One of `MULTI_CLASS` or `MULTI_LABEL`.
     * `MULTI_CLASS` is also known as "Single Label" in the AWS Console.
     */
    mode?: pulumi.Input<string>;
    /**
     * KMS Key used to encrypt trained Document Classifiers.
     * Can be a KMS Key ID or a KMS Key ARN.
     */
    modelKmsKeyId?: pulumi.Input<string>;
    /**
     * Name for the Document Classifier.
     * Has a maximum length of 63 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Configuration for the output results of training.
     * See the `outputDataConfig` Configuration Block section below.
     */
    outputDataConfig?: pulumi.Input<inputs.comprehend.DocumentClassifierOutputDataConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` Configuration Block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Name for the version of the Document Classifier.
     * Each version must have a unique name within the Document Classifier.
     * If omitted, the provider will assign a random, unique version name.
     * If explicitly set to `""`, no version name will be set.
     * Has a maximum length of 63 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     * Conflicts with `versionNamePrefix`.
     */
    versionName?: pulumi.Input<string>;
    /**
     * Creates a unique version name beginning with the specified prefix.
     * Has a maximum length of 37 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     * Conflicts with `versionName`.
     */
    versionNamePrefix?: pulumi.Input<string>;
    /**
     * KMS Key used to encrypt storage volumes during job processing.
     * Can be a KMS Key ID or a KMS Key ARN.
     */
    volumeKmsKeyId?: pulumi.Input<string>;
    /**
     * Configuration parameters for VPC to contain Document Classifier resources.
     * See the `vpcConfig` Configuration Block section below.
     */
    vpcConfig?: pulumi.Input<inputs.comprehend.DocumentClassifierVpcConfig>;
}
/**
 * The set of arguments for constructing a DocumentClassifier resource.
 */
export interface DocumentClassifierArgs {
    /**
     * The ARN for an IAM Role which allows Comprehend to read the training and testing data.
     */
    dataAccessRoleArn: pulumi.Input<string>;
    /**
     * Configuration for the training and testing data.
     * See the `inputDataConfig` Configuration Block section below.
     */
    inputDataConfig: pulumi.Input<inputs.comprehend.DocumentClassifierInputDataConfig>;
    /**
     * Two-letter language code for the language.
     * One of `en`, `es`, `fr`, `it`, `de`, or `pt`.
     */
    languageCode: pulumi.Input<string>;
    /**
     * The document classification mode.
     * One of `MULTI_CLASS` or `MULTI_LABEL`.
     * `MULTI_CLASS` is also known as "Single Label" in the AWS Console.
     */
    mode?: pulumi.Input<string>;
    /**
     * KMS Key used to encrypt trained Document Classifiers.
     * Can be a KMS Key ID or a KMS Key ARN.
     */
    modelKmsKeyId?: pulumi.Input<string>;
    /**
     * Name for the Document Classifier.
     * Has a maximum length of 63 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Configuration for the output results of training.
     * See the `outputDataConfig` Configuration Block section below.
     */
    outputDataConfig?: pulumi.Input<inputs.comprehend.DocumentClassifierOutputDataConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` Configuration Block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name for the version of the Document Classifier.
     * Each version must have a unique name within the Document Classifier.
     * If omitted, the provider will assign a random, unique version name.
     * If explicitly set to `""`, no version name will be set.
     * Has a maximum length of 63 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     * Conflicts with `versionNamePrefix`.
     */
    versionName?: pulumi.Input<string>;
    /**
     * Creates a unique version name beginning with the specified prefix.
     * Has a maximum length of 37 characters.
     * Can contain upper- and lower-case letters, numbers, and hypen (`-`).
     * Conflicts with `versionName`.
     */
    versionNamePrefix?: pulumi.Input<string>;
    /**
     * KMS Key used to encrypt storage volumes during job processing.
     * Can be a KMS Key ID or a KMS Key ARN.
     */
    volumeKmsKeyId?: pulumi.Input<string>;
    /**
     * Configuration parameters for VPC to contain Document Classifier resources.
     * See the `vpcConfig` Configuration Block section below.
     */
    vpcConfig?: pulumi.Input<inputs.comprehend.DocumentClassifierVpcConfig>;
}
