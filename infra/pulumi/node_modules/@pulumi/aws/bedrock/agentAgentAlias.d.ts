import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Agents for Amazon Bedrock Agent Alias.
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
 *     idleTtl: 500,
 *     foundationModel: "anthropic.claude-v2",
 * });
 * const exampleAgentAgentAlias = new aws.bedrock.AgentAgentAlias("example", {
 *     agentAliasName: "my-agent-alias",
 *     agentId: exampleAgentAgent.agentId,
 *     description: "Test Alias",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Agents for Amazon Bedrock Agent Alias using the alias ID and the agent ID separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentAgentAlias:AgentAgentAlias example 66IVY0GUTF,GGRRAED6JP
 * ```
 */
export declare class AgentAgentAlias extends pulumi.CustomResource {
    /**
     * Get an existing AgentAgentAlias resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentAgentAliasState, opts?: pulumi.CustomResourceOptions): AgentAgentAlias;
    /**
     * Returns true if the given object is an instance of AgentAgentAlias.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentAgentAlias;
    /**
     * ARN of the alias.
     */
    readonly agentAliasArn: pulumi.Output<string>;
    /**
     * Unique identifier of the alias.
     */
    readonly agentAliasId: pulumi.Output<string>;
    /**
     * Name of the alias.
     */
    readonly agentAliasName: pulumi.Output<string>;
    /**
     * Identifier of the agent to create an alias for.
     *
     * The following arguments are optional:
     */
    readonly agentId: pulumi.Output<string>;
    /**
     * Description of the alias.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Details about the routing configuration of the alias. See `routingConfiguration` Block for details.
     */
    readonly routingConfigurations: pulumi.Output<outputs.bedrock.AgentAgentAliasRoutingConfiguration[]>;
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentAgentAliasTimeouts | undefined>;
    /**
     * Create a AgentAgentAlias resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentAgentAliasArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentAgentAlias resources.
 */
export interface AgentAgentAliasState {
    /**
     * ARN of the alias.
     */
    agentAliasArn?: pulumi.Input<string>;
    /**
     * Unique identifier of the alias.
     */
    agentAliasId?: pulumi.Input<string>;
    /**
     * Name of the alias.
     */
    agentAliasName?: pulumi.Input<string>;
    /**
     * Identifier of the agent to create an alias for.
     *
     * The following arguments are optional:
     */
    agentId?: pulumi.Input<string>;
    /**
     * Description of the alias.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details about the routing configuration of the alias. See `routingConfiguration` Block for details.
     */
    routingConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentAliasRoutingConfiguration>[]>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentAliasTimeouts>;
}
/**
 * The set of arguments for constructing a AgentAgentAlias resource.
 */
export interface AgentAgentAliasArgs {
    /**
     * Name of the alias.
     */
    agentAliasName: pulumi.Input<string>;
    /**
     * Identifier of the agent to create an alias for.
     *
     * The following arguments are optional:
     */
    agentId: pulumi.Input<string>;
    /**
     * Description of the alias.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details about the routing configuration of the alias. See `routingConfiguration` Block for details.
     */
    routingConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentAgentAliasRoutingConfiguration>[]>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentAliasTimeouts>;
}
