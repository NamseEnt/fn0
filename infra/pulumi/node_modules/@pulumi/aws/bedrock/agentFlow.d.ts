import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Bedrock Agents Flow.
 *
 * ## Example Usage
 *
 * The default definition:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentFlow("example", {
 *     name: "example",
 *     executionRoleArn: exampleAwsIamRole.arn,
 *     definition: {
 *         connections: [
 *             {
 *                 name: "FlowInputNodeFlowInputNode0ToPrompt_1PromptsNode0",
 *                 source: "FlowInputNode",
 *                 target: "Prompt_1",
 *                 type: "Data",
 *                 configuration: {
 *                     data: {
 *                         sourceOutput: "document",
 *                         targetInput: "topic",
 *                     },
 *                 },
 *             },
 *             {
 *                 name: "Prompt_1PromptsNode0ToFlowOutputNodeFlowOutputNode0",
 *                 source: "Prompt_1",
 *                 target: "FlowOutputNode",
 *                 type: "Data",
 *                 configuration: {
 *                     data: {
 *                         sourceOutput: "modelCompletion",
 *                         targetInput: "document",
 *                     },
 *                 },
 *             },
 *         ],
 *         nodes: [
 *             {
 *                 name: "FlowInputNode",
 *                 type: "Input",
 *                 configuration: {
 *                     input: {},
 *                 },
 *                 outputs: [{
 *                     name: "document",
 *                     type: "String",
 *                 }],
 *             },
 *             {
 *                 name: "Prompt_1",
 *                 type: "Prompt",
 *                 configuration: {
 *                     prompt: {
 *                         sourceConfiguration: {
 *                             inline: {
 *                                 modelId: "amazon.titan-text-express-v1",
 *                                 templateType: "TEXT",
 *                                 inferenceConfiguration: {
 *                                     text: {
 *                                         maxTokens: 2048,
 *                                         stopSequences: ["User:"],
 *                                         temperature: 0,
 *                                         topP: 0.8999999761581421,
 *                                     },
 *                                 },
 *                                 templateConfiguration: {
 *                                     text: {
 *                                         text: "Write a paragraph about {{topic}}.",
 *                                         inputVariables: [{
 *                                             name: "topic",
 *                                         }],
 *                                     },
 *                                 },
 *                             },
 *                         },
 *                     },
 *                 },
 *                 inputs: [{
 *                     expression: "$.data",
 *                     name: "topic",
 *                     type: "String",
 *                 }],
 *                 outputs: [{
 *                     name: "modelCompletion",
 *                     type: "String",
 *                 }],
 *             },
 *             {
 *                 name: "FlowOutputNode",
 *                 type: "Output",
 *                 configuration: {
 *                     output: {},
 *                 },
 *                 inputs: [{
 *                     expression: "$.data",
 *                     name: "document",
 *                     type: "String",
 *                 }],
 *             },
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock Agents Flow using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentFlow:AgentFlow example ABCDEFGHIJ
 * ```
 */
export declare class AgentFlow extends pulumi.CustomResource {
    /**
     * Get an existing AgentFlow resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentFlowState, opts?: pulumi.CustomResourceOptions): AgentFlow;
    /**
     * Returns true if the given object is an instance of AgentFlow.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentFlow;
    /**
     * The Amazon Resource Name (ARN) of the flow.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The time at which the flow was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the KMS key to encrypt the flow.
     */
    readonly customerEncryptionKeyArn: pulumi.Output<string | undefined>;
    /**
     * A definition of the nodes and connections between nodes in the flow. See Definition for more information.
     */
    readonly definition: pulumi.Output<outputs.bedrock.AgentFlowDefinition | undefined>;
    /**
     * A description for the flow.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the service role with permissions to create and manage a flow. For more information, see [Create a service role for flows in Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/userguide/flows-permissions.html) in the Amazon Bedrock User Guide.
     *
     * The following arguments are optional:
     */
    readonly executionRoleArn: pulumi.Output<string>;
    /**
     * A name for the flow.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The status of the flow.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentFlowTimeouts | undefined>;
    /**
     * The time at which the flow was last updated.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * The version of the flow.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a AgentFlow resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentFlowArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentFlow resources.
 */
export interface AgentFlowState {
    /**
     * The Amazon Resource Name (ARN) of the flow.
     */
    arn?: pulumi.Input<string>;
    /**
     * The time at which the flow was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the KMS key to encrypt the flow.
     */
    customerEncryptionKeyArn?: pulumi.Input<string>;
    /**
     * A definition of the nodes and connections between nodes in the flow. See Definition for more information.
     */
    definition?: pulumi.Input<inputs.bedrock.AgentFlowDefinition>;
    /**
     * A description for the flow.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the service role with permissions to create and manage a flow. For more information, see [Create a service role for flows in Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/userguide/flows-permissions.html) in the Amazon Bedrock User Guide.
     *
     * The following arguments are optional:
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * A name for the flow.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of the flow.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentFlowTimeouts>;
    /**
     * The time at which the flow was last updated.
     */
    updatedAt?: pulumi.Input<string>;
    /**
     * The version of the flow.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AgentFlow resource.
 */
export interface AgentFlowArgs {
    /**
     * The Amazon Resource Name (ARN) of the KMS key to encrypt the flow.
     */
    customerEncryptionKeyArn?: pulumi.Input<string>;
    /**
     * A definition of the nodes and connections between nodes in the flow. See Definition for more information.
     */
    definition?: pulumi.Input<inputs.bedrock.AgentFlowDefinition>;
    /**
     * A description for the flow.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the service role with permissions to create and manage a flow. For more information, see [Create a service role for flows in Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/userguide/flows-permissions.html) in the Amazon Bedrock User Guide.
     *
     * The following arguments are optional:
     */
    executionRoleArn: pulumi.Input<string>;
    /**
     * A name for the flow.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentFlowTimeouts>;
}
