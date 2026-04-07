import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Agents for Amazon Bedrock Agent Knowledge Base Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentAgentKnowledgeBaseAssociation("example", {
 *     agentId: "GGRRAED6JP",
 *     description: "Example Knowledge base",
 *     knowledgeBaseId: "EMDPPAYPZI",
 *     knowledgeBaseState: "ENABLED",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Agents for Amazon Bedrock Agent Knowledge Base Association using the agent ID, the agent version, and the knowledge base ID separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentAgentKnowledgeBaseAssociation:AgentAgentKnowledgeBaseAssociation example GGRRAED6JP,DRAFT,EMDPPAYPZI
 * ```
 */
export declare class AgentAgentKnowledgeBaseAssociation extends pulumi.CustomResource {
    /**
     * Get an existing AgentAgentKnowledgeBaseAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentAgentKnowledgeBaseAssociationState, opts?: pulumi.CustomResourceOptions): AgentAgentKnowledgeBaseAssociation;
    /**
     * Returns true if the given object is an instance of AgentAgentKnowledgeBaseAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentAgentKnowledgeBaseAssociation;
    /**
     * Unique identifier of the agent with which you want to associate the knowledge base.
     */
    readonly agentId: pulumi.Output<string>;
    /**
     * Version of the agent with which you want to associate the knowledge base. Valid values: `DRAFT`.
     */
    readonly agentVersion: pulumi.Output<string>;
    /**
     * Description of what the agent should use the knowledge base for.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Unique identifier of the knowledge base to associate with the agent.
     */
    readonly knowledgeBaseId: pulumi.Output<string>;
    /**
     * Whether to use the knowledge base when sending an [InvokeAgent](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent-runtime_InvokeAgent.html) request. Valid values: `ENABLED`, `DISABLED`.
     *
     * The following arguments are optional:
     */
    readonly knowledgeBaseState: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentAgentKnowledgeBaseAssociationTimeouts | undefined>;
    /**
     * Create a AgentAgentKnowledgeBaseAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentAgentKnowledgeBaseAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentAgentKnowledgeBaseAssociation resources.
 */
export interface AgentAgentKnowledgeBaseAssociationState {
    /**
     * Unique identifier of the agent with which you want to associate the knowledge base.
     */
    agentId?: pulumi.Input<string>;
    /**
     * Version of the agent with which you want to associate the knowledge base. Valid values: `DRAFT`.
     */
    agentVersion?: pulumi.Input<string>;
    /**
     * Description of what the agent should use the knowledge base for.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier of the knowledge base to associate with the agent.
     */
    knowledgeBaseId?: pulumi.Input<string>;
    /**
     * Whether to use the knowledge base when sending an [InvokeAgent](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent-runtime_InvokeAgent.html) request. Valid values: `ENABLED`, `DISABLED`.
     *
     * The following arguments are optional:
     */
    knowledgeBaseState?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentKnowledgeBaseAssociationTimeouts>;
}
/**
 * The set of arguments for constructing a AgentAgentKnowledgeBaseAssociation resource.
 */
export interface AgentAgentKnowledgeBaseAssociationArgs {
    /**
     * Unique identifier of the agent with which you want to associate the knowledge base.
     */
    agentId: pulumi.Input<string>;
    /**
     * Version of the agent with which you want to associate the knowledge base. Valid values: `DRAFT`.
     */
    agentVersion?: pulumi.Input<string>;
    /**
     * Description of what the agent should use the knowledge base for.
     */
    description: pulumi.Input<string>;
    /**
     * Unique identifier of the knowledge base to associate with the agent.
     */
    knowledgeBaseId: pulumi.Input<string>;
    /**
     * Whether to use the knowledge base when sending an [InvokeAgent](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent-runtime_InvokeAgent.html) request. Valid values: `ENABLED`, `DISABLED`.
     *
     * The following arguments are optional:
     */
    knowledgeBaseState: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentKnowledgeBaseAssociationTimeouts>;
}
