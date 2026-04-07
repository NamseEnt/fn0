import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Chatbot Slack Workspace.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.chatbot.getSlackWorkspace({
 *     slackTeamName: "abc",
 * });
 * ```
 */
export declare function getSlackWorkspace(args: GetSlackWorkspaceArgs, opts?: pulumi.InvokeOptions): Promise<GetSlackWorkspaceResult>;
/**
 * A collection of arguments for invoking getSlackWorkspace.
 */
export interface GetSlackWorkspaceArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Slack workspace name configured with AWS Chatbot.
     */
    slackTeamName: string;
}
/**
 * A collection of values returned by getSlackWorkspace.
 */
export interface GetSlackWorkspaceResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * ID of the Slack Workspace assigned by AWS Chatbot.
     */
    readonly slackTeamId: string;
    readonly slackTeamName: string;
}
/**
 * Data source for managing an AWS Chatbot Slack Workspace.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.chatbot.getSlackWorkspace({
 *     slackTeamName: "abc",
 * });
 * ```
 */
export declare function getSlackWorkspaceOutput(args: GetSlackWorkspaceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSlackWorkspaceResult>;
/**
 * A collection of arguments for invoking getSlackWorkspace.
 */
export interface GetSlackWorkspaceOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Slack workspace name configured with AWS Chatbot.
     */
    slackTeamName: pulumi.Input<string>;
}
