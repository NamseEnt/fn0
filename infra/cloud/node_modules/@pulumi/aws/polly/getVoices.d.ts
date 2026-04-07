import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Polly Voices.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.polly.getVoices({});
 * ```
 *
 * ### With Language Code
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.polly.getVoices({
 *     languageCode: "en-GB",
 * });
 * ```
 */
export declare function getVoices(args?: GetVoicesArgs, opts?: pulumi.InvokeOptions): Promise<GetVoicesResult>;
/**
 * A collection of arguments for invoking getVoices.
 */
export interface GetVoicesArgs {
    /**
     * Engine used by Amazon Polly when processing input text for speech synthesis. Valid values are `standard`, `neural`, and `long-form`.
     */
    engine?: string;
    /**
     * Whether to return any bilingual voices that use the specified language as an additional language.
     */
    includeAdditionalLanguageCodes?: boolean;
    /**
     * Language identification tag for filtering the list of voices returned. If not specified, all available voices are returned.
     */
    languageCode?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * List of voices with their properties. See `voices` Attribute Reference below.
     */
    voices?: inputs.polly.GetVoicesVoice[];
}
/**
 * A collection of values returned by getVoices.
 */
export interface GetVoicesResult {
    readonly engine?: string;
    /**
     * Amazon Polly assigned voice ID.
     */
    readonly id: string;
    readonly includeAdditionalLanguageCodes?: boolean;
    /**
     * Language code of the voice.
     */
    readonly languageCode?: string;
    readonly region: string;
    /**
     * List of voices with their properties. See `voices` Attribute Reference below.
     */
    readonly voices?: outputs.polly.GetVoicesVoice[];
}
/**
 * Data source for managing an AWS Polly Voices.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.polly.getVoices({});
 * ```
 *
 * ### With Language Code
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.polly.getVoices({
 *     languageCode: "en-GB",
 * });
 * ```
 */
export declare function getVoicesOutput(args?: GetVoicesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVoicesResult>;
/**
 * A collection of arguments for invoking getVoices.
 */
export interface GetVoicesOutputArgs {
    /**
     * Engine used by Amazon Polly when processing input text for speech synthesis. Valid values are `standard`, `neural`, and `long-form`.
     */
    engine?: pulumi.Input<string>;
    /**
     * Whether to return any bilingual voices that use the specified language as an additional language.
     */
    includeAdditionalLanguageCodes?: pulumi.Input<boolean>;
    /**
     * Language identification tag for filtering the list of voices returned. If not specified, all available voices are returned.
     */
    languageCode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of voices with their properties. See `voices` Attribute Reference below.
     */
    voices?: pulumi.Input<pulumi.Input<inputs.polly.GetVoicesVoiceArgs>[]>;
}
