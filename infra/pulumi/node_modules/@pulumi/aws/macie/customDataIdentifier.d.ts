import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an [AWS Macie Custom Data Identifier](https://docs.aws.amazon.com/macie/latest/APIReference/custom-data-identifiers-id.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.macie2.Account("example", {});
 * const exampleCustomDataIdentifier = new aws.macie.CustomDataIdentifier("example", {
 *     name: "NAME OF CUSTOM DATA IDENTIFIER",
 *     regex: "[0-9]{3}-[0-9]{2}-[0-9]{4}",
 *     description: "DESCRIPTION",
 *     maximumMatchDistance: 10,
 *     keywords: ["keyword"],
 *     ignoreWords: ["ignore"],
 * }, {
 *     dependsOn: [test],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_macie2_custom_data_identifier` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:macie/customDataIdentifier:CustomDataIdentifier example abcd1
 * ```
 */
export declare class CustomDataIdentifier extends pulumi.CustomResource {
    /**
     * Get an existing CustomDataIdentifier resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomDataIdentifierState, opts?: pulumi.CustomResourceOptions): CustomDataIdentifier;
    /**
     * Returns true if the given object is an instance of CustomDataIdentifier.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomDataIdentifier;
    /**
     * The Amazon Resource Name (ARN) of the custom data identifier.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date and time, in UTC and extended RFC 3339 format, when the Amazon Macie account was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * A custom description of the custom data identifier. The description can contain as many as 512 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * An array that lists specific character sequences (ignore words) to exclude from the results. If the text matched by the regular expression is the same as any string in this array, Amazon Macie ignores it. The array can contain as many as 10 ignore words. Each ignore word can contain 4 - 90 characters. Ignore words are case sensitive.
     */
    readonly ignoreWords: pulumi.Output<string[] | undefined>;
    /**
     * An array that lists specific character sequences (keywords), one of which must be within proximity (`maximumMatchDistance`) of the regular expression to match. The array can contain as many as 50 keywords. Each keyword can contain 3 - 90 characters. Keywords aren't case sensitive.
     */
    readonly keywords: pulumi.Output<string[] | undefined>;
    /**
     * The maximum number of characters that can exist between text that matches the regex pattern and the character sequences specified by the keywords array. Macie includes or excludes a result based on the proximity of a keyword to text that matches the regex pattern. The distance can be 1 - 300 characters. The default value is 50.
     */
    readonly maximumMatchDistance: pulumi.Output<number>;
    /**
     * A custom name for the custom data identifier. The name can contain as many as 128 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * The regular expression (regex) that defines the pattern to match. The expression can contain as many as 512 characters.
     */
    readonly regex: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a CustomDataIdentifier resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: CustomDataIdentifierArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomDataIdentifier resources.
 */
export interface CustomDataIdentifierState {
    /**
     * The Amazon Resource Name (ARN) of the custom data identifier.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date and time, in UTC and extended RFC 3339 format, when the Amazon Macie account was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * A custom description of the custom data identifier. The description can contain as many as 512 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * An array that lists specific character sequences (ignore words) to exclude from the results. If the text matched by the regular expression is the same as any string in this array, Amazon Macie ignores it. The array can contain as many as 10 ignore words. Each ignore word can contain 4 - 90 characters. Ignore words are case sensitive.
     */
    ignoreWords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * An array that lists specific character sequences (keywords), one of which must be within proximity (`maximumMatchDistance`) of the regular expression to match. The array can contain as many as 50 keywords. Each keyword can contain 3 - 90 characters. Keywords aren't case sensitive.
     */
    keywords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The maximum number of characters that can exist between text that matches the regex pattern and the character sequences specified by the keywords array. Macie includes or excludes a result based on the proximity of a keyword to text that matches the regex pattern. The distance can be 1 - 300 characters. The default value is 50.
     */
    maximumMatchDistance?: pulumi.Input<number>;
    /**
     * A custom name for the custom data identifier. The name can contain as many as 128 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The regular expression (regex) that defines the pattern to match. The expression can contain as many as 512 characters.
     */
    regex?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a CustomDataIdentifier resource.
 */
export interface CustomDataIdentifierArgs {
    /**
     * A custom description of the custom data identifier. The description can contain as many as 512 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * An array that lists specific character sequences (ignore words) to exclude from the results. If the text matched by the regular expression is the same as any string in this array, Amazon Macie ignores it. The array can contain as many as 10 ignore words. Each ignore word can contain 4 - 90 characters. Ignore words are case sensitive.
     */
    ignoreWords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * An array that lists specific character sequences (keywords), one of which must be within proximity (`maximumMatchDistance`) of the regular expression to match. The array can contain as many as 50 keywords. Each keyword can contain 3 - 90 characters. Keywords aren't case sensitive.
     */
    keywords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The maximum number of characters that can exist between text that matches the regex pattern and the character sequences specified by the keywords array. Macie includes or excludes a result based on the proximity of a keyword to text that matches the regex pattern. The distance can be 1 - 300 characters. The default value is 50.
     */
    maximumMatchDistance?: pulumi.Input<number>;
    /**
     * A custom name for the custom data identifier. The name can contain as many as 128 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The regular expression (regex) that defines the pattern to match. The expression can contain as many as 512 characters.
     */
    regex?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
