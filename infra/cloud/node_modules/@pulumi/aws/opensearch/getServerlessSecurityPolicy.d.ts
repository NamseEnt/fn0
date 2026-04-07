import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get information about an AWS OpenSearch Serverless Security Policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.opensearch.getServerlessSecurityPolicy({
 *     name: "example-security-policy",
 *     type: "encryption",
 * });
 * ```
 */
export declare function getServerlessSecurityPolicy(args: GetServerlessSecurityPolicyArgs, opts?: pulumi.InvokeOptions): Promise<GetServerlessSecurityPolicyResult>;
/**
 * A collection of arguments for invoking getServerlessSecurityPolicy.
 */
export interface GetServerlessSecurityPolicyArgs {
    /**
     * Name of the policy
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Type of security policy. One of `encryption` or `network`.
     */
    type: string;
}
/**
 * A collection of values returned by getServerlessSecurityPolicy.
 */
export interface GetServerlessSecurityPolicyResult {
    /**
     * The date the security policy was created.
     */
    readonly createdDate: string;
    /**
     * Description of the security policy.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The date the security policy was last modified.
     */
    readonly lastModifiedDate: string;
    readonly name: string;
    /**
     * The JSON policy document without any whitespaces.
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
 * Use this data source to get information about an AWS OpenSearch Serverless Security Policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.opensearch.getServerlessSecurityPolicy({
 *     name: "example-security-policy",
 *     type: "encryption",
 * });
 * ```
 */
export declare function getServerlessSecurityPolicyOutput(args: GetServerlessSecurityPolicyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetServerlessSecurityPolicyResult>;
/**
 * A collection of arguments for invoking getServerlessSecurityPolicy.
 */
export interface GetServerlessSecurityPolicyOutputArgs {
    /**
     * Name of the policy
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of security policy. One of `encryption` or `network`.
     */
    type: pulumi.Input<string>;
}
