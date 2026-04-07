import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS OpenSearch Serverless Lifecycle Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.opensearch.getServerlessLifecyclePolicy({
 *     name: "example-lifecycle-policy",
 *     type: "retention",
 * });
 * ```
 */
export declare function getServerlessLifecyclePolicy(args: GetServerlessLifecyclePolicyArgs, opts?: pulumi.InvokeOptions): Promise<GetServerlessLifecyclePolicyResult>;
/**
 * A collection of arguments for invoking getServerlessLifecyclePolicy.
 */
export interface GetServerlessLifecyclePolicyArgs {
    /**
     * Name of the policy
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Type of lifecycle policy. Must be `retention`.
     */
    type: string;
}
/**
 * A collection of values returned by getServerlessLifecyclePolicy.
 */
export interface GetServerlessLifecyclePolicyResult {
    /**
     * The date the lifecycle policy was created.
     */
    readonly createdDate: string;
    /**
     * Description of the policy. Typically used to store information about the permissions defined in the policy.
     */
    readonly description: string;
    readonly id: string;
    /**
     * The date the lifecycle policy was last modified.
     */
    readonly lastModifiedDate: string;
    readonly name: string;
    /**
     * JSON policy document to use as the content for the new policy.
     */
    readonly policy: string;
    /**
     * Version of the policy.
     */
    readonly policyVersion: string;
    readonly region: string;
    readonly type: string;
}
/**
 * Data source for managing an AWS OpenSearch Serverless Lifecycle Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.opensearch.getServerlessLifecyclePolicy({
 *     name: "example-lifecycle-policy",
 *     type: "retention",
 * });
 * ```
 */
export declare function getServerlessLifecyclePolicyOutput(args: GetServerlessLifecyclePolicyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetServerlessLifecyclePolicyResult>;
/**
 * A collection of arguments for invoking getServerlessLifecyclePolicy.
 */
export interface GetServerlessLifecyclePolicyOutputArgs {
    /**
     * Name of the policy
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of lifecycle policy. Must be `retention`.
     */
    type: pulumi.Input<string>;
}
