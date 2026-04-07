import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Bedrock AgentCore Workload Identity. Workload Identity provides OAuth2-based authentication and authorization for AI agents to access external resources securely.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreWorkloadIdentity("example", {
 *     name: "example-workload-identity",
 *     allowedResourceOauth2ReturnUrls: ["https://example.com/callback"],
 * });
 * ```
 *
 * ### Workload Identity with Multiple Return URLs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreWorkloadIdentity("example", {
 *     name: "example-workload-identity",
 *     allowedResourceOauth2ReturnUrls: [
 *         "https://app.example.com/oauth/callback",
 *         "https://api.example.com/auth/return",
 *         "https://example.com/callback",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Workload Identity using the workload identity name. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreWorkloadIdentity:AgentcoreWorkloadIdentity example example-workload-identity
 * ```
 */
export declare class AgentcoreWorkloadIdentity extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreWorkloadIdentity resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreWorkloadIdentityState, opts?: pulumi.CustomResourceOptions): AgentcoreWorkloadIdentity;
    /**
     * Returns true if the given object is an instance of AgentcoreWorkloadIdentity.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreWorkloadIdentity;
    /**
     * Set of allowed OAuth2 return URLs for resources associated with this workload identity. These URLs are used as valid redirect targets during OAuth2 authentication flows.
     */
    readonly allowedResourceOauth2ReturnUrls: pulumi.Output<string[] | undefined>;
    /**
     * Name of the workload identity. Must be 3-255 characters and contain only alphanumeric characters, hyphens, periods, and underscores.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the Workload Identity.
     */
    readonly workloadIdentityArn: pulumi.Output<string>;
    /**
     * Create a AgentcoreWorkloadIdentity resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AgentcoreWorkloadIdentityArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreWorkloadIdentity resources.
 */
export interface AgentcoreWorkloadIdentityState {
    /**
     * Set of allowed OAuth2 return URLs for resources associated with this workload identity. These URLs are used as valid redirect targets during OAuth2 authentication flows.
     */
    allowedResourceOauth2ReturnUrls?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the workload identity. Must be 3-255 characters and contain only alphanumeric characters, hyphens, periods, and underscores.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the Workload Identity.
     */
    workloadIdentityArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AgentcoreWorkloadIdentity resource.
 */
export interface AgentcoreWorkloadIdentityArgs {
    /**
     * Set of allowed OAuth2 return URLs for resources associated with this workload identity. These URLs are used as valid redirect targets during OAuth2 authentication flows.
     */
    allowedResourceOauth2ReturnUrls?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the workload identity. Must be 3-255 characters and contain only alphanumeric characters, hyphens, periods, and underscores.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
