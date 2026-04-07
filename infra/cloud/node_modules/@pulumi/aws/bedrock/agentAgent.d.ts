import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Agents for Amazon Bedrock Agent.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const currentGetRegion = aws.getRegion({});
 * const exampleAgentTrust = Promise.all([current, currentGetPartition, currentGetRegion, current]).then(([current, currentGetPartition, currentGetRegion, current1]) => aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             identifiers: ["bedrock.amazonaws.com"],
 *             type: "Service",
 *         }],
 *         conditions: [
 *             {
 *                 test: "StringEquals",
 *                 values: [current.accountId],
 *                 variable: "aws:SourceAccount",
 *             },
 *             {
 *                 test: "ArnLike",
 *                 values: [`arn:${currentGetPartition.partition}:bedrock:${currentGetRegion.region}:${current1.accountId}:agent/*`],
 *                 variable: "AWS:SourceArn",
 *             },
 *         ],
 *     }],
 * }));
 * const exampleAgentPermissions = Promise.all([currentGetPartition, currentGetRegion]).then(([currentGetPartition, currentGetRegion]) => aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["bedrock:InvokeModel"],
 *         resources: [`arn:${currentGetPartition.partition}:bedrock:${currentGetRegion.region}::foundation-model/anthropic.claude-v2`],
 *     }],
 * }));
 * const example = new aws.iam.Role("example", {
 *     assumeRolePolicy: exampleAgentTrust.then(exampleAgentTrust => exampleAgentTrust.json),
 *     namePrefix: "AmazonBedrockExecutionRoleForAgents_",
 * });
 * const exampleRolePolicy = new aws.iam.RolePolicy("example", {
 *     policy: exampleAgentPermissions.then(exampleAgentPermissions => exampleAgentPermissions.json),
 *     role: example.id,
 * });
 * const exampleAgentAgent = new aws.bedrock.AgentAgent("example", {
 *     agentName: "my-agent-name",
 *     agentResourceRoleArn: example.arn,
 *     idleSessionTtlInSeconds: 500,
 *     instruction: "You are a friendly assistant who helps answer questions.",
 *     foundationModel: "anthropic.claude-v2",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Agents for Amazon Bedrock Agent using the agent ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentAgent:AgentAgent example GGRRAED6JP
 * ```
 */
export declare class AgentAgent extends pulumi.CustomResource {
    /**
     * Get an existing AgentAgent resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentAgentState, opts?: pulumi.CustomResourceOptions): AgentAgent;
    /**
     * Returns true if the given object is an instance of AgentAgent.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentAgent;
    /**
     * ARN of the agent.
     */
    readonly agentArn: pulumi.Output<string>;
    /**
     * Agents collaboration role. Valid values: `SUPERVISOR`, `SUPERVISOR_ROUTER`, `DISABLED`.
     */
    readonly agentCollaboration: pulumi.Output<string>;
    /**
     * Unique identifier of the agent.
     */
    readonly agentId: pulumi.Output<string>;
    /**
     * Name of the agent.
     */
    readonly agentName: pulumi.Output<string>;
    /**
     * ARN of the IAM role with permissions to invoke API operations on the agent.
     */
    readonly agentResourceRoleArn: pulumi.Output<string>;
    /**
     * Version of the agent.
     */
    readonly agentVersion: pulumi.Output<string>;
    /**
     * ARN of the AWS KMS key that encrypts the agent.
     */
    readonly customerEncryptionKeyArn: pulumi.Output<string | undefined>;
    /**
     * Description of the agent.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Foundation model used for orchestration by the agent.
     *
     * The following arguments are optional:
     */
    readonly foundationModel: pulumi.Output<string>;
    /**
     * Details about the guardrail associated with the agent. See `guardrailConfiguration` Block for details.
     */
    readonly guardrailConfigurations: pulumi.Output<outputs.bedrock.AgentAgentGuardrailConfiguration[] | undefined>;
    /**
     * Number of seconds for which Amazon Bedrock keeps information about a user's conversation with the agent. A user interaction remains active for the amount of time specified. If no conversation occurs during this time, the session expires and Amazon Bedrock deletes any data provided before the timeout.
     */
    readonly idleSessionTtlInSeconds: pulumi.Output<number>;
    /**
     * Instructions that tell the agent what it should do and how it should interact with users. If `prepareAgent` is `true` this argument is required. The valid range is 40 - 20000 characters.
     */
    readonly instruction: pulumi.Output<string>;
    /**
     * Configurations for the agent's ability to retain the conversational context.
     */
    readonly memoryConfigurations: pulumi.Output<outputs.bedrock.AgentAgentMemoryConfiguration[]>;
    /**
     * Whether to prepare the agent after creation or modification. Defaults to `true`.
     */
    readonly prepareAgent: pulumi.Output<boolean>;
    /**
     * Timestamp of when the agent was last prepared.
     */
    readonly preparedAt: pulumi.Output<string>;
    /**
     * Configurations to override prompt templates in different parts of an agent sequence. For more information, see [Advanced prompts](https://docs.aws.amazon.com/bedrock/latest/userguide/advanced-prompts.html). See `promptOverrideConfiguration` Block for details.
     */
    readonly promptOverrideConfigurations: pulumi.Output<outputs.bedrock.AgentAgentPromptOverrideConfiguration[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether the in-use check is skipped when deleting the agent.
     */
    readonly skipResourceInUseCheck: pulumi.Output<boolean>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentAgentTimeouts | undefined>;
    /**
     * Create a AgentAgent resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentAgentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentAgent resources.
 */
export interface AgentAgentState {
    /**
     * ARN of the agent.
     */
    agentArn?: pulumi.Input<string>;
    /**
     * Agents collaboration role. Valid values: `SUPERVISOR`, `SUPERVISOR_ROUTER`, `DISABLED`.
     */
    agentCollaboration?: pulumi.Input<string>;
    /**
     * Unique identifier of the agent.
     */
    agentId?: pulumi.Input<string>;
    /**
     * Name of the agent.
     */
    agentName?: pulumi.Input<string>;
    /**
     * ARN of the IAM role with permissions to invoke API operations on the agent.
     */
    agentResourceRoleArn?: pulumi.Input<string>;
    /**
     * Version of the agent.
     */
    agentVersion?: pulumi.Input<string>;
    /**
     * ARN of the AWS KMS key that encrypts the agent.
     */
    customerEncryptionKeyArn?: pulumi.Input<string>;
    /**
     * Description of the agent.
     */
    description?: pulumi.Input<string>;
    /**
     * Foundation model used for orchestration by the agent.
     *
     * The following arguments are optional:
     */
    foundationModel?: pulumi.Input<string>;
    /**
     * Details about the guardrail associated with the agent. See `guardrailConfiguration` Block for details.
     */
    guardrailConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentGuardrailConfiguration>[]>;
    /**
     * Number of seconds for which Amazon Bedrock keeps information about a user's conversation with the agent. A user interaction remains active for the amount of time specified. If no conversation occurs during this time, the session expires and Amazon Bedrock deletes any data provided before the timeout.
     */
    idleSessionTtlInSeconds?: pulumi.Input<number>;
    /**
     * Instructions that tell the agent what it should do and how it should interact with users. If `prepareAgent` is `true` this argument is required. The valid range is 40 - 20000 characters.
     */
    instruction?: pulumi.Input<string>;
    /**
     * Configurations for the agent's ability to retain the conversational context.
     */
    memoryConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentMemoryConfiguration>[]>;
    /**
     * Whether to prepare the agent after creation or modification. Defaults to `true`.
     */
    prepareAgent?: pulumi.Input<boolean>;
    /**
     * Timestamp of when the agent was last prepared.
     */
    preparedAt?: pulumi.Input<string>;
    /**
     * Configurations to override prompt templates in different parts of an agent sequence. For more information, see [Advanced prompts](https://docs.aws.amazon.com/bedrock/latest/userguide/advanced-prompts.html). See `promptOverrideConfiguration` Block for details.
     */
    promptOverrideConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentPromptOverrideConfiguration>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether the in-use check is skipped when deleting the agent.
     */
    skipResourceInUseCheck?: pulumi.Input<boolean>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentTimeouts>;
}
/**
 * The set of arguments for constructing a AgentAgent resource.
 */
export interface AgentAgentArgs {
    /**
     * Agents collaboration role. Valid values: `SUPERVISOR`, `SUPERVISOR_ROUTER`, `DISABLED`.
     */
    agentCollaboration?: pulumi.Input<string>;
    /**
     * Name of the agent.
     */
    agentName: pulumi.Input<string>;
    /**
     * ARN of the IAM role with permissions to invoke API operations on the agent.
     */
    agentResourceRoleArn: pulumi.Input<string>;
    /**
     * ARN of the AWS KMS key that encrypts the agent.
     */
    customerEncryptionKeyArn?: pulumi.Input<string>;
    /**
     * Description of the agent.
     */
    description?: pulumi.Input<string>;
    /**
     * Foundation model used for orchestration by the agent.
     *
     * The following arguments are optional:
     */
    foundationModel: pulumi.Input<string>;
    /**
     * Details about the guardrail associated with the agent. See `guardrailConfiguration` Block for details.
     */
    guardrailConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentGuardrailConfiguration>[]>;
    /**
     * Number of seconds for which Amazon Bedrock keeps information about a user's conversation with the agent. A user interaction remains active for the amount of time specified. If no conversation occurs during this time, the session expires and Amazon Bedrock deletes any data provided before the timeout.
     */
    idleSessionTtlInSeconds?: pulumi.Input<number>;
    /**
     * Instructions that tell the agent what it should do and how it should interact with users. If `prepareAgent` is `true` this argument is required. The valid range is 40 - 20000 characters.
     */
    instruction?: pulumi.Input<string>;
    /**
     * Configurations for the agent's ability to retain the conversational context.
     */
    memoryConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentMemoryConfiguration>[]>;
    /**
     * Whether to prepare the agent after creation or modification. Defaults to `true`.
     */
    prepareAgent?: pulumi.Input<boolean>;
    /**
     * Configurations to override prompt templates in different parts of an agent sequence. For more information, see [Advanced prompts](https://docs.aws.amazon.com/bedrock/latest/userguide/advanced-prompts.html). See `promptOverrideConfiguration` Block for details.
     */
    promptOverrideConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentPromptOverrideConfiguration>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether the in-use check is skipped when deleting the agent.
     */
    skipResourceInUseCheck?: pulumi.Input<boolean>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentTimeouts>;
}
