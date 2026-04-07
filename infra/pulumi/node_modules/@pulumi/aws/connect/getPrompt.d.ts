import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about a specific Amazon Connect Prompt.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getPrompt({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Beep.wav",
 * });
 * ```
 */
export declare function getPrompt(args: GetPromptArgs, opts?: pulumi.InvokeOptions): Promise<GetPromptResult>;
/**
 * A collection of arguments for invoking getPrompt.
 */
export interface GetPromptArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: string;
    /**
     * Returns information on a specific Prompt by name
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPrompt.
 */
export interface GetPromptResult {
    /**
     * ARN of the Prompt.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceId: string;
    readonly name: string;
    /**
     * Identifier for the prompt.
     */
    readonly promptId: string;
    readonly region: string;
}
/**
 * Provides details about a specific Amazon Connect Prompt.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getPrompt({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Beep.wav",
 * });
 * ```
 */
export declare function getPromptOutput(args: GetPromptOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPromptResult>;
/**
 * A collection of arguments for invoking getPrompt.
 */
export interface GetPromptOutputArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: pulumi.Input<string>;
    /**
     * Returns information on a specific Prompt by name
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
