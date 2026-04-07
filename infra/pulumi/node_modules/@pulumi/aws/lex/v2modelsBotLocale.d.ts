import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Lex V2 Models Bot Locale.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lex.V2modelsBotLocale("example", {
 *     botId: exampleAwsLexv2modelsBot.id,
 *     botVersion: "DRAFT",
 *     localeId: "en_US",
 *     nLuIntentConfidenceThreshold: 0.7,
 * });
 * ```
 *
 * ### Voice Settings
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lex.V2modelsBotLocale("example", {
 *     botId: exampleAwsLexv2modelsBot.id,
 *     botVersion: "DRAFT",
 *     localeId: "en_US",
 *     nLuIntentConfidenceThreshold: 0.7,
 *     voiceSettings: {
 *         voiceId: "Kendra",
 *         engine: "standard",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lex V2 Models Bot Locale using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:lex/v2modelsBotLocale:V2modelsBotLocale example en_US,abcd-12345678,1
 * ```
 */
export declare class V2modelsBotLocale extends pulumi.CustomResource {
    /**
     * Get an existing V2modelsBotLocale resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: V2modelsBotLocaleState, opts?: pulumi.CustomResourceOptions): V2modelsBotLocale;
    /**
     * Returns true if the given object is an instance of V2modelsBotLocale.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is V2modelsBotLocale;
    /**
     * Identifier of the bot to create the locale for.
     */
    readonly botId: pulumi.Output<string>;
    /**
     * Version of the bot to create the locale for. This can only be the draft version of the bot.
     */
    readonly botVersion: pulumi.Output<string>;
    /**
     * Description of the bot locale. Use this to help identify the bot locale in lists.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Identifier of the language and locale that the bot will be used in. The string must match one of the supported locales. All of the intents, slot types, and slots used in the bot must have the same locale. For more information, see Supported languages (https://docs.aws.amazon.com/lexv2/latest/dg/how-languages.html)
     */
    readonly localeId: pulumi.Output<string>;
    /**
     * Determines the threshold where Amazon Lex will insert the AMAZON.FallbackIntent, AMAZON.KendraSearchIntent, or both when returning alternative intents.
     *
     * The following arguments are optional:
     */
    readonly nLuIntentConfidenceThreshold: pulumi.Output<number>;
    /**
     * Specified locale name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.lex.V2modelsBotLocaleTimeouts | undefined>;
    /**
     * Amazon Polly voice ID that Amazon Lex uses for voice interaction with the user. See `voiceSettings`.
     */
    readonly voiceSettings: pulumi.Output<outputs.lex.V2modelsBotLocaleVoiceSettings | undefined>;
    /**
     * Create a V2modelsBotLocale resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: V2modelsBotLocaleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering V2modelsBotLocale resources.
 */
export interface V2modelsBotLocaleState {
    /**
     * Identifier of the bot to create the locale for.
     */
    botId?: pulumi.Input<string>;
    /**
     * Version of the bot to create the locale for. This can only be the draft version of the bot.
     */
    botVersion?: pulumi.Input<string>;
    /**
     * Description of the bot locale. Use this to help identify the bot locale in lists.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of the language and locale that the bot will be used in. The string must match one of the supported locales. All of the intents, slot types, and slots used in the bot must have the same locale. For more information, see Supported languages (https://docs.aws.amazon.com/lexv2/latest/dg/how-languages.html)
     */
    localeId?: pulumi.Input<string>;
    /**
     * Determines the threshold where Amazon Lex will insert the AMAZON.FallbackIntent, AMAZON.KendraSearchIntent, or both when returning alternative intents.
     *
     * The following arguments are optional:
     */
    nLuIntentConfidenceThreshold?: pulumi.Input<number>;
    /**
     * Specified locale name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.lex.V2modelsBotLocaleTimeouts>;
    /**
     * Amazon Polly voice ID that Amazon Lex uses for voice interaction with the user. See `voiceSettings`.
     */
    voiceSettings?: pulumi.Input<inputs.lex.V2modelsBotLocaleVoiceSettings>;
}
/**
 * The set of arguments for constructing a V2modelsBotLocale resource.
 */
export interface V2modelsBotLocaleArgs {
    /**
     * Identifier of the bot to create the locale for.
     */
    botId: pulumi.Input<string>;
    /**
     * Version of the bot to create the locale for. This can only be the draft version of the bot.
     */
    botVersion: pulumi.Input<string>;
    /**
     * Description of the bot locale. Use this to help identify the bot locale in lists.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of the language and locale that the bot will be used in. The string must match one of the supported locales. All of the intents, slot types, and slots used in the bot must have the same locale. For more information, see Supported languages (https://docs.aws.amazon.com/lexv2/latest/dg/how-languages.html)
     */
    localeId: pulumi.Input<string>;
    /**
     * Determines the threshold where Amazon Lex will insert the AMAZON.FallbackIntent, AMAZON.KendraSearchIntent, or both when returning alternative intents.
     *
     * The following arguments are optional:
     */
    nLuIntentConfidenceThreshold: pulumi.Input<number>;
    /**
     * Specified locale name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.lex.V2modelsBotLocaleTimeouts>;
    /**
     * Amazon Polly voice ID that Amazon Lex uses for voice interaction with the user. See `voiceSettings`.
     */
    voiceSettings?: pulumi.Input<inputs.lex.V2modelsBotLocaleVoiceSettings>;
}
