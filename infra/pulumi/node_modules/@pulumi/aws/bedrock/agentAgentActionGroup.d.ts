import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Agents for Amazon Bedrock Agent Action Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.bedrock.AgentAgentActionGroup("example", {
 *     actionGroupName: "example",
 *     agentId: "GGRRAED6JP",
 *     agentVersion: "DRAFT",
 *     skipResourceInUseCheck: true,
 *     actionGroupExecutor: {
 *         lambda: "arn:aws:lambda:us-west-2:123456789012:function:example-function",
 *     },
 *     apiSchema: {
 *         payload: std.file({
 *             input: "path/to/schema.yaml",
 *         }).then(invoke => invoke.result),
 *     },
 * });
 * ```
 *
 * ### API Schema in S3 Bucket
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentAgentActionGroup("example", {
 *     actionGroupName: "example",
 *     agentId: "GGRRAED6JP",
 *     agentVersion: "DRAFT",
 *     skipResourceInUseCheck: true,
 *     actionGroupExecutor: {
 *         lambda: "arn:aws:lambda:us-west-2:123456789012:function:example-function",
 *     },
 *     apiSchema: {
 *         s3: {
 *             s3BucketName: "example-bucket",
 *             s3ObjectKey: "path/to/schema.json",
 *         },
 *     },
 * });
 * ```
 *
 * ### Function Schema (Simplified Schema)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentAgentActionGroup("example", {
 *     actionGroupName: "example",
 *     agentId: "GGRRAED6JP",
 *     agentVersion: "DRAFT",
 *     skipResourceInUseCheck: true,
 *     actionGroupExecutor: {
 *         lambda: "arn:aws:lambda:us-west-2:123456789012:function:example-function",
 *     },
 *     functionSchema: {
 *         memberFunctions: {
 *             functions: [{
 *                 name: "example-function",
 *                 description: "Example function",
 *                 parameters: [
 *                     {
 *                         mapBlockKey: "param1",
 *                         type: "string",
 *                         description: "The first parameter",
 *                         required: true,
 *                     },
 *                     {
 *                         mapBlockKey: "param2",
 *                         type: "integer",
 *                         description: "The second parameter",
 *                         required: false,
 *                     },
 *                 ],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### Return of Control
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.bedrock.AgentAgentActionGroup("example", {
 *     actionGroupName: "example",
 *     agentId: "GGRRAED6JP",
 *     agentVersion: "DRAFT",
 *     skipResourceInUseCheck: true,
 *     actionGroupExecutor: {
 *         customControl: "RETURN_CONTROL",
 *     },
 *     apiSchema: {
 *         payload: std.file({
 *             input: "path/to/schema.yaml",
 *         }).then(invoke => invoke.result),
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Agents for Amazon Bedrock Agent Action Group the action group ID, the agent ID, and the agent version separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentAgentActionGroup:AgentAgentActionGroup example MMAUDBZTH4,GGRRAED6JP,DRAFT
 * ```
 */
export declare class AgentAgentActionGroup extends pulumi.CustomResource {
    /**
     * Get an existing AgentAgentActionGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentAgentActionGroupState, opts?: pulumi.CustomResourceOptions): AgentAgentActionGroup;
    /**
     * Returns true if the given object is an instance of AgentAgentActionGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentAgentActionGroup;
    /**
     * ARN of the Lambda function containing the business logic that is carried out upon invoking the action or custom control method for handling the information elicited from the user. See `actionGroupExecutor` Block for details.
     *
     * The following arguments are optional:
     */
    readonly actionGroupExecutor: pulumi.Output<outputs.bedrock.AgentAgentActionGroupActionGroupExecutor | undefined>;
    /**
     * Unique identifier of the action group.
     */
    readonly actionGroupId: pulumi.Output<string>;
    /**
     * Name of the action group.
     */
    readonly actionGroupName: pulumi.Output<string>;
    /**
     * Whether the action group is available for the agent to invoke or not when sending an [InvokeAgent](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent-runtime_InvokeAgent.html) request. Valid values: `ENABLED`, `DISABLED`.
     */
    readonly actionGroupState: pulumi.Output<string>;
    /**
     * The unique identifier of the agent for which to create the action group.
     */
    readonly agentId: pulumi.Output<string>;
    /**
     * Version of the agent for which to create the action group. Valid values: `DRAFT`.
     */
    readonly agentVersion: pulumi.Output<string>;
    /**
     * Either details about the S3 object containing the OpenAPI schema for the action group or the JSON or YAML-formatted payload defining the schema. For more information, see [Action group OpenAPI schemas](https://docs.aws.amazon.com/bedrock/latest/userguide/agents-api-schema.html). See `apiSchema` Block for details.
     */
    readonly apiSchema: pulumi.Output<outputs.bedrock.AgentAgentActionGroupApiSchema | undefined>;
    /**
     * Description of the action group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Describes the function schema for the action group.
     * Each function represents an action in an action group.
     * See `functionSchema` Block for details.
     */
    readonly functionSchema: pulumi.Output<outputs.bedrock.AgentAgentActionGroupFunctionSchema | undefined>;
    /**
     * To allow your agent to request the user for additional information when trying to complete a task, set this argument to `AMAZON.UserInput`. You must leave the `description`, `apiSchema`, and `actionGroupExecutor` arguments blank for this action group. Valid values: `AMAZON.UserInput`.
     */
    readonly parentActionGroupSignature: pulumi.Output<string | undefined>;
    /**
     * Whether or not to prepare the agent after creation or modification. Defaults to `true`.
     */
    readonly prepareAgent: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether the in-use check is skipped when deleting the action group.
     */
    readonly skipResourceInUseCheck: pulumi.Output<boolean>;
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentAgentActionGroupTimeouts | undefined>;
    /**
     * Create a AgentAgentActionGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentAgentActionGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentAgentActionGroup resources.
 */
export interface AgentAgentActionGroupState {
    /**
     * ARN of the Lambda function containing the business logic that is carried out upon invoking the action or custom control method for handling the information elicited from the user. See `actionGroupExecutor` Block for details.
     *
     * The following arguments are optional:
     */
    actionGroupExecutor?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupActionGroupExecutor>;
    /**
     * Unique identifier of the action group.
     */
    actionGroupId?: pulumi.Input<string>;
    /**
     * Name of the action group.
     */
    actionGroupName?: pulumi.Input<string>;
    /**
     * Whether the action group is available for the agent to invoke or not when sending an [InvokeAgent](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent-runtime_InvokeAgent.html) request. Valid values: `ENABLED`, `DISABLED`.
     */
    actionGroupState?: pulumi.Input<string>;
    /**
     * The unique identifier of the agent for which to create the action group.
     */
    agentId?: pulumi.Input<string>;
    /**
     * Version of the agent for which to create the action group. Valid values: `DRAFT`.
     */
    agentVersion?: pulumi.Input<string>;
    /**
     * Either details about the S3 object containing the OpenAPI schema for the action group or the JSON or YAML-formatted payload defining the schema. For more information, see [Action group OpenAPI schemas](https://docs.aws.amazon.com/bedrock/latest/userguide/agents-api-schema.html). See `apiSchema` Block for details.
     */
    apiSchema?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupApiSchema>;
    /**
     * Description of the action group.
     */
    description?: pulumi.Input<string>;
    /**
     * Describes the function schema for the action group.
     * Each function represents an action in an action group.
     * See `functionSchema` Block for details.
     */
    functionSchema?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupFunctionSchema>;
    /**
     * To allow your agent to request the user for additional information when trying to complete a task, set this argument to `AMAZON.UserInput`. You must leave the `description`, `apiSchema`, and `actionGroupExecutor` arguments blank for this action group. Valid values: `AMAZON.UserInput`.
     */
    parentActionGroupSignature?: pulumi.Input<string>;
    /**
     * Whether or not to prepare the agent after creation or modification. Defaults to `true`.
     */
    prepareAgent?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether the in-use check is skipped when deleting the action group.
     */
    skipResourceInUseCheck?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupTimeouts>;
}
/**
 * The set of arguments for constructing a AgentAgentActionGroup resource.
 */
export interface AgentAgentActionGroupArgs {
    /**
     * ARN of the Lambda function containing the business logic that is carried out upon invoking the action or custom control method for handling the information elicited from the user. See `actionGroupExecutor` Block for details.
     *
     * The following arguments are optional:
     */
    actionGroupExecutor?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupActionGroupExecutor>;
    /**
     * Name of the action group.
     */
    actionGroupName: pulumi.Input<string>;
    /**
     * Whether the action group is available for the agent to invoke or not when sending an [InvokeAgent](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_agent-runtime_InvokeAgent.html) request. Valid values: `ENABLED`, `DISABLED`.
     */
    actionGroupState?: pulumi.Input<string>;
    /**
     * The unique identifier of the agent for which to create the action group.
     */
    agentId: pulumi.Input<string>;
    /**
     * Version of the agent for which to create the action group. Valid values: `DRAFT`.
     */
    agentVersion: pulumi.Input<string>;
    /**
     * Either details about the S3 object containing the OpenAPI schema for the action group or the JSON or YAML-formatted payload defining the schema. For more information, see [Action group OpenAPI schemas](https://docs.aws.amazon.com/bedrock/latest/userguide/agents-api-schema.html). See `apiSchema` Block for details.
     */
    apiSchema?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupApiSchema>;
    /**
     * Description of the action group.
     */
    description?: pulumi.Input<string>;
    /**
     * Describes the function schema for the action group.
     * Each function represents an action in an action group.
     * See `functionSchema` Block for details.
     */
    functionSchema?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupFunctionSchema>;
    /**
     * To allow your agent to request the user for additional information when trying to complete a task, set this argument to `AMAZON.UserInput`. You must leave the `description`, `apiSchema`, and `actionGroupExecutor` arguments blank for this action group. Valid values: `AMAZON.UserInput`.
     */
    parentActionGroupSignature?: pulumi.Input<string>;
    /**
     * Whether or not to prepare the agent after creation or modification. Defaults to `true`.
     */
    prepareAgent?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether the in-use check is skipped when deleting the action group.
     */
    skipResourceInUseCheck?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentAgentActionGroupTimeouts>;
}
