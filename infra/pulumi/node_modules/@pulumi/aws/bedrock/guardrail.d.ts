import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon Bedrock Guardrail.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.Guardrail("example", {
 *     name: "example",
 *     blockedInputMessaging: "example",
 *     blockedOutputsMessaging: "example",
 *     description: "example",
 *     contentPolicyConfig: {
 *         filtersConfigs: [{
 *             inputStrength: "MEDIUM",
 *             outputStrength: "MEDIUM",
 *             type: "HATE",
 *         }],
 *         tierConfigs: [{
 *             tierName: "STANDARD",
 *         }],
 *     },
 *     sensitiveInformationPolicyConfig: {
 *         piiEntitiesConfigs: [{
 *             action: "BLOCK",
 *             inputAction: "BLOCK",
 *             outputAction: "ANONYMIZE",
 *             inputEnabled: true,
 *             outputEnabled: true,
 *             type: "NAME",
 *         }],
 *         regexesConfigs: [{
 *             action: "BLOCK",
 *             inputAction: "BLOCK",
 *             outputAction: "BLOCK",
 *             inputEnabled: true,
 *             outputEnabled: false,
 *             description: "example regex",
 *             name: "regex_example",
 *             pattern: "^\\d{3}-\\d{2}-\\d{4}$",
 *         }],
 *     },
 *     topicPolicyConfig: {
 *         topicsConfigs: [{
 *             name: "investment_topic",
 *             examples: ["Where should I invest my money ?"],
 *             type: "DENY",
 *             definition: "Investment advice refers to inquiries, guidance, or recommendations regarding the management or allocation of funds or assets with the goal of generating returns .",
 *         }],
 *         tierConfigs: [{
 *             tierName: "CLASSIC",
 *         }],
 *     },
 *     wordPolicyConfig: {
 *         managedWordListsConfigs: [{
 *             type: "PROFANITY",
 *         }],
 *         wordsConfigs: [{
 *             text: "HATE",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Bedrock Guardrail using using a comma-delimited string of `guardrail_id` and `version`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/guardrail:Guardrail example guardrail-id-12345678,DRAFT
 * ```
 */
export declare class Guardrail extends pulumi.CustomResource {
    /**
     * Get an existing Guardrail resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GuardrailState, opts?: pulumi.CustomResourceOptions): Guardrail;
    /**
     * Returns true if the given object is an instance of Guardrail.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Guardrail;
    /**
     * Message to return when the guardrail blocks a prompt.
     */
    readonly blockedInputMessaging: pulumi.Output<string>;
    /**
     * Message to return when the guardrail blocks a model response.
     */
    readonly blockedOutputsMessaging: pulumi.Output<string>;
    /**
     * Content policy config for a guardrail. See Content Policy Config for more information.
     */
    readonly contentPolicyConfig: pulumi.Output<outputs.bedrock.GuardrailContentPolicyConfig | undefined>;
    /**
     * Contextual grounding policy config for a guardrail. See Contextual Grounding Policy Config for more information.
     */
    readonly contextualGroundingPolicyConfig: pulumi.Output<outputs.bedrock.GuardrailContextualGroundingPolicyConfig | undefined>;
    /**
     * Unix epoch timestamp in seconds for when the Guardrail was created.
     */
    readonly createdAt: pulumi.Output<string>;
    readonly crossRegionConfig: pulumi.Output<outputs.bedrock.GuardrailCrossRegionConfig | undefined>;
    /**
     * Description of the guardrail or its version.
     */
    readonly description: pulumi.Output<string>;
    /**
     * ARN of the Guardrail.
     */
    readonly guardrailArn: pulumi.Output<string>;
    /**
     * ID of the Guardrail.
     */
    readonly guardrailId: pulumi.Output<string>;
    /**
     * The KMS key with which the guardrail was encrypted at rest.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Name of the guardrail.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Sensitive information policy config for a guardrail. See Sensitive Information Policy Config for more information.
     */
    readonly sensitiveInformationPolicyConfig: pulumi.Output<outputs.bedrock.GuardrailSensitiveInformationPolicyConfig | undefined>;
    /**
     * Status of the Bedrock Guardrail. One of `READY`, `FAILED`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.bedrock.GuardrailTimeouts | undefined>;
    /**
     * Topic policy config for a guardrail. See Topic Policy Config for more information.
     */
    readonly topicPolicyConfig: pulumi.Output<outputs.bedrock.GuardrailTopicPolicyConfig | undefined>;
    /**
     * Version of the Guardrail.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Word policy config for a guardrail. See Word Policy Config for more information.
     */
    readonly wordPolicyConfig: pulumi.Output<outputs.bedrock.GuardrailWordPolicyConfig | undefined>;
    /**
     * Create a Guardrail resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GuardrailArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Guardrail resources.
 */
export interface GuardrailState {
    /**
     * Message to return when the guardrail blocks a prompt.
     */
    blockedInputMessaging?: pulumi.Input<string>;
    /**
     * Message to return when the guardrail blocks a model response.
     */
    blockedOutputsMessaging?: pulumi.Input<string>;
    /**
     * Content policy config for a guardrail. See Content Policy Config for more information.
     */
    contentPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailContentPolicyConfig>;
    /**
     * Contextual grounding policy config for a guardrail. See Contextual Grounding Policy Config for more information.
     */
    contextualGroundingPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailContextualGroundingPolicyConfig>;
    /**
     * Unix epoch timestamp in seconds for when the Guardrail was created.
     */
    createdAt?: pulumi.Input<string>;
    crossRegionConfig?: pulumi.Input<inputs.bedrock.GuardrailCrossRegionConfig>;
    /**
     * Description of the guardrail or its version.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the Guardrail.
     */
    guardrailArn?: pulumi.Input<string>;
    /**
     * ID of the Guardrail.
     */
    guardrailId?: pulumi.Input<string>;
    /**
     * The KMS key with which the guardrail was encrypted at rest.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the guardrail.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Sensitive information policy config for a guardrail. See Sensitive Information Policy Config for more information.
     */
    sensitiveInformationPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailSensitiveInformationPolicyConfig>;
    /**
     * Status of the Bedrock Guardrail. One of `READY`, `FAILED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.GuardrailTimeouts>;
    /**
     * Topic policy config for a guardrail. See Topic Policy Config for more information.
     */
    topicPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailTopicPolicyConfig>;
    /**
     * Version of the Guardrail.
     */
    version?: pulumi.Input<string>;
    /**
     * Word policy config for a guardrail. See Word Policy Config for more information.
     */
    wordPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailWordPolicyConfig>;
}
/**
 * The set of arguments for constructing a Guardrail resource.
 */
export interface GuardrailArgs {
    /**
     * Message to return when the guardrail blocks a prompt.
     */
    blockedInputMessaging: pulumi.Input<string>;
    /**
     * Message to return when the guardrail blocks a model response.
     */
    blockedOutputsMessaging: pulumi.Input<string>;
    /**
     * Content policy config for a guardrail. See Content Policy Config for more information.
     */
    contentPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailContentPolicyConfig>;
    /**
     * Contextual grounding policy config for a guardrail. See Contextual Grounding Policy Config for more information.
     */
    contextualGroundingPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailContextualGroundingPolicyConfig>;
    crossRegionConfig?: pulumi.Input<inputs.bedrock.GuardrailCrossRegionConfig>;
    /**
     * Description of the guardrail or its version.
     */
    description?: pulumi.Input<string>;
    /**
     * The KMS key with which the guardrail was encrypted at rest.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the guardrail.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Sensitive information policy config for a guardrail. See Sensitive Information Policy Config for more information.
     */
    sensitiveInformationPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailSensitiveInformationPolicyConfig>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.GuardrailTimeouts>;
    /**
     * Topic policy config for a guardrail. See Topic Policy Config for more information.
     */
    topicPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailTopicPolicyConfig>;
    /**
     * Word policy config for a guardrail. See Word Policy Config for more information.
     */
    wordPolicyConfig?: pulumi.Input<inputs.bedrock.GuardrailWordPolicyConfig>;
}
