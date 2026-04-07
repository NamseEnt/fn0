import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Transcribe VocabularyFilter.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transcribe.VocabularyFilter("example", {
 *     vocabularyFilterName: "example",
 *     languageCode: "en-US",
 *     words: [
 *         "cars",
 *         "bucket",
 *     ],
 *     tags: {
 *         tag1: "value1",
 *         tag2: "value3",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transcribe VocabularyFilter using the `vocabulary_filter_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:transcribe/vocabularyFilter:VocabularyFilter example example-name
 * ```
 */
export declare class VocabularyFilter extends pulumi.CustomResource {
    /**
     * Get an existing VocabularyFilter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VocabularyFilterState, opts?: pulumi.CustomResourceOptions): VocabularyFilter;
    /**
     * Returns true if the given object is an instance of VocabularyFilter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VocabularyFilter;
    /**
     * ARN of the VocabularyFilter.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Generated download URI.
     */
    readonly downloadUri: pulumi.Output<string>;
    /**
     * The language code you selected for your vocabulary filter. Refer to the [supported languages](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html) page for accepted codes.
     */
    readonly languageCode: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the VocabularyFilter. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The Amazon S3 location (URI) of the text file that contains your custom VocabularyFilter. Conflicts with `words` argument.
     */
    readonly vocabularyFilterFileUri: pulumi.Output<string | undefined>;
    /**
     * The name of the VocabularyFilter.
     *
     * The following arguments are optional:
     */
    readonly vocabularyFilterName: pulumi.Output<string>;
    /**
     * A list of terms to include in the vocabulary. Conflicts with `vocabularyFilterFileUri` argument.
     */
    readonly words: pulumi.Output<string[] | undefined>;
    /**
     * Create a VocabularyFilter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VocabularyFilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VocabularyFilter resources.
 */
export interface VocabularyFilterState {
    /**
     * ARN of the VocabularyFilter.
     */
    arn?: pulumi.Input<string>;
    /**
     * Generated download URI.
     */
    downloadUri?: pulumi.Input<string>;
    /**
     * The language code you selected for your vocabulary filter. Refer to the [supported languages](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html) page for accepted codes.
     */
    languageCode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the VocabularyFilter. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon S3 location (URI) of the text file that contains your custom VocabularyFilter. Conflicts with `words` argument.
     */
    vocabularyFilterFileUri?: pulumi.Input<string>;
    /**
     * The name of the VocabularyFilter.
     *
     * The following arguments are optional:
     */
    vocabularyFilterName?: pulumi.Input<string>;
    /**
     * A list of terms to include in the vocabulary. Conflicts with `vocabularyFilterFileUri` argument.
     */
    words?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a VocabularyFilter resource.
 */
export interface VocabularyFilterArgs {
    /**
     * The language code you selected for your vocabulary filter. Refer to the [supported languages](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html) page for accepted codes.
     */
    languageCode: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the VocabularyFilter. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon S3 location (URI) of the text file that contains your custom VocabularyFilter. Conflicts with `words` argument.
     */
    vocabularyFilterFileUri?: pulumi.Input<string>;
    /**
     * The name of the VocabularyFilter.
     *
     * The following arguments are optional:
     */
    vocabularyFilterName: pulumi.Input<string>;
    /**
     * A list of terms to include in the vocabulary. Conflicts with `vocabularyFilterFileUri` argument.
     */
    words?: pulumi.Input<pulumi.Input<string>[]>;
}
