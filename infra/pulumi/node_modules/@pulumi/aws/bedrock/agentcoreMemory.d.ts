import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Bedrock AgentCore Memory. Memory provides persistent storage for AI agent interactions, allowing agents to retain context across conversations and sessions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["bedrock-agentcore.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const example = new aws.iam.Role("example", {
 *     name: "bedrock-agentcore-memory-role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("example", {
 *     role: example.name,
 *     policyArn: "arn:aws:iam::aws:policy/AmazonBedrockAgentCoreMemoryBedrockModelInferenceExecutionRolePolicy",
 * });
 * const exampleAgentcoreMemory = new aws.bedrock.AgentcoreMemory("example", {
 *     name: "example_memory",
 *     eventExpiryDuration: 30,
 * });
 * ```
 *
 * ### Memory with Custom Encryption and Role
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {description: "KMS key for Bedrock AgentCore Memory"});
 * const exampleAgentcoreMemory = new aws.bedrock.AgentcoreMemory("example", {
 *     name: "example_memory",
 *     description: "Memory for customer service agent",
 *     eventExpiryDuration: 60,
 *     encryptionKeyArn: example.arn,
 *     memoryExecutionRoleArn: exampleAwsIamRole.arn,
 *     clientToken: "unique-client-token",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Memory using the memory ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreMemory:AgentcoreMemory example MEMORY1234567890
 * ```
 */
export declare class AgentcoreMemory extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreMemory resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreMemoryState, opts?: pulumi.CustomResourceOptions): AgentcoreMemory;
    /**
     * Returns true if the given object is an instance of AgentcoreMemory.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreMemory;
    /**
     * ARN of the Memory.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the memory.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the KMS key used to encrypt the memory. If not provided, AWS managed encryption is used.
     */
    readonly encryptionKeyArn: pulumi.Output<string | undefined>;
    /**
     * Number of days after which memory events expire. Must be a positive integer in the range of 7 to 365.
     *
     * The following arguments are optional:
     */
    readonly eventExpiryDuration: pulumi.Output<number>;
    /**
     * ARN of the IAM role that the memory service assumes to perform operations. Required when using custom memory strategies with model processing.
     */
    readonly memoryExecutionRoleArn: pulumi.Output<string | undefined>;
    /**
     * Name of the memory.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentcoreMemoryTimeouts | undefined>;
    /**
     * Create a AgentcoreMemory resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentcoreMemoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreMemory resources.
 */
export interface AgentcoreMemoryState {
    /**
     * ARN of the Memory.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the memory.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the memory. If not provided, AWS managed encryption is used.
     */
    encryptionKeyArn?: pulumi.Input<string>;
    /**
     * Number of days after which memory events expire. Must be a positive integer in the range of 7 to 365.
     *
     * The following arguments are optional:
     */
    eventExpiryDuration?: pulumi.Input<number>;
    /**
     * ARN of the IAM role that the memory service assumes to perform operations. Required when using custom memory strategies with model processing.
     */
    memoryExecutionRoleArn?: pulumi.Input<string>;
    /**
     * Name of the memory.
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
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreMemoryTimeouts>;
}
/**
 * The set of arguments for constructing a AgentcoreMemory resource.
 */
export interface AgentcoreMemoryArgs {
    /**
     * Description of the memory.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the memory. If not provided, AWS managed encryption is used.
     */
    encryptionKeyArn?: pulumi.Input<string>;
    /**
     * Number of days after which memory events expire. Must be a positive integer in the range of 7 to 365.
     *
     * The following arguments are optional:
     */
    eventExpiryDuration: pulumi.Input<number>;
    /**
     * ARN of the IAM role that the memory service assumes to perform operations. Required when using custom memory strategies with model processing.
     */
    memoryExecutionRoleArn?: pulumi.Input<string>;
    /**
     * Name of the memory.
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreMemoryTimeouts>;
}
