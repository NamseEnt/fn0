import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Transcribe Vocabulary.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {
 *     bucket: "example-vocab-123",
 *     forceDestroy: true,
 * });
 * const object = new aws.s3.BucketObjectv2("object", {
 *     bucket: example.id,
 *     key: "transcribe/test1.txt",
 *     source: new pulumi.asset.FileAsset("test.txt"),
 * });
 * const exampleVocabulary = new aws.transcribe.Vocabulary("example", {
 *     vocabularyName: "example",
 *     languageCode: "en-US",
 *     vocabularyFileUri: pulumi.interpolate`s3://${example.id}/${object.key}`,
 *     tags: {
 *         tag1: "value1",
 *         tag2: "value3",
 *     },
 * }, {
 *     dependsOn: [object],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transcribe Vocabulary using the `vocabulary_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:transcribe/vocabulary:Vocabulary example example-name
 * ```
 */
export declare class Vocabulary extends pulumi.CustomResource {
    /**
     * Get an existing Vocabulary resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VocabularyState, opts?: pulumi.CustomResourceOptions): Vocabulary;
    /**
     * Returns true if the given object is an instance of Vocabulary.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Vocabulary;
    /**
     * ARN of the Vocabulary.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Generated download URI.
     */
    readonly downloadUri: pulumi.Output<string>;
    /**
     * The language code you selected for your vocabulary.
     */
    readonly languageCode: pulumi.Output<string>;
    /**
     * A list of terms to include in the vocabulary. Conflicts with `vocabularyFileUri`
     */
    readonly phrases: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the Vocabulary. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The Amazon S3 location (URI) of the text file that contains your custom vocabulary. Conflicts wth `phrases`.
     */
    readonly vocabularyFileUri: pulumi.Output<string>;
    /**
     * The name of the Vocabulary.
     *
     * The following arguments are optional:
     */
    readonly vocabularyName: pulumi.Output<string>;
    /**
     * Create a Vocabulary resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VocabularyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Vocabulary resources.
 */
export interface VocabularyState {
    /**
     * ARN of the Vocabulary.
     */
    arn?: pulumi.Input<string>;
    /**
     * Generated download URI.
     */
    downloadUri?: pulumi.Input<string>;
    /**
     * The language code you selected for your vocabulary.
     */
    languageCode?: pulumi.Input<string>;
    /**
     * A list of terms to include in the vocabulary. Conflicts with `vocabularyFileUri`
     */
    phrases?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the Vocabulary. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon S3 location (URI) of the text file that contains your custom vocabulary. Conflicts wth `phrases`.
     */
    vocabularyFileUri?: pulumi.Input<string>;
    /**
     * The name of the Vocabulary.
     *
     * The following arguments are optional:
     */
    vocabularyName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Vocabulary resource.
 */
export interface VocabularyArgs {
    /**
     * The language code you selected for your vocabulary.
     */
    languageCode: pulumi.Input<string>;
    /**
     * A list of terms to include in the vocabulary. Conflicts with `vocabularyFileUri`
     */
    phrases?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the Vocabulary. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon S3 location (URI) of the text file that contains your custom vocabulary. Conflicts wth `phrases`.
     */
    vocabularyFileUri?: pulumi.Input<string>;
    /**
     * The name of the Vocabulary.
     *
     * The following arguments are optional:
     */
    vocabularyName: pulumi.Input<string>;
}
