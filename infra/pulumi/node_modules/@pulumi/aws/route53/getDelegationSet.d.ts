import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.route53.DelegationSet` provides details about a specific Route 53 Delegation Set.
 *
 * This data source allows to find a list of name servers associated with a specific delegation set.
 *
 * ## Example Usage
 *
 * The following example shows how to get a delegation set from its id.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const dset = aws.route53.getDelegationSet({
 *     id: "MQWGHCBFAKEID",
 * });
 * ```
 */
export declare function getDelegationSet(args: GetDelegationSetArgs, opts?: pulumi.InvokeOptions): Promise<GetDelegationSetResult>;
/**
 * A collection of arguments for invoking getDelegationSet.
 */
export interface GetDelegationSetArgs {
    /**
     * Delegation set ID.
     */
    id: string;
}
/**
 * A collection of values returned by getDelegationSet.
 */
export interface GetDelegationSetResult {
    /**
     * ARN of the Delegation Set.
     */
    readonly arn: string;
    /**
     * Caller Reference of the delegation set.
     */
    readonly callerReference: string;
    readonly id: string;
    /**
     * List of DNS name servers for the delegation set.
     */
    readonly nameServers: string[];
}
/**
 * `aws.route53.DelegationSet` provides details about a specific Route 53 Delegation Set.
 *
 * This data source allows to find a list of name servers associated with a specific delegation set.
 *
 * ## Example Usage
 *
 * The following example shows how to get a delegation set from its id.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const dset = aws.route53.getDelegationSet({
 *     id: "MQWGHCBFAKEID",
 * });
 * ```
 */
export declare function getDelegationSetOutput(args: GetDelegationSetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDelegationSetResult>;
/**
 * A collection of arguments for invoking getDelegationSet.
 */
export interface GetDelegationSetOutputArgs {
    /**
     * Delegation set ID.
     */
    id: pulumi.Input<string>;
}
