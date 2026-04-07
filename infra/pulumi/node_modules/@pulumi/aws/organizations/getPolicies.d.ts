import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Organizations Policies.
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
 * const example = aws.organizations.getPolicies({
 *     filter: "SERVICE_CONTROL_POLICY",
 * });
 * const exampleGetPolicy = example.then(example => std.toset({
 *     input: example.ids,
 * })).then(invoke => .reduce((__obj, [__key, __value]) => ({ ...__obj, [__key]: aws.organizations.getPolicy({
 *     policyId: __value,
 * }) })));
 * ```
 */
export declare function getPolicies(args: GetPoliciesArgs, opts?: pulumi.InvokeOptions): Promise<GetPoliciesResult>;
/**
 * A collection of arguments for invoking getPolicies.
 */
export interface GetPoliciesArgs {
    /**
     * The type of policies to be returned in the response. Valid values are `AISERVICES_OPT_OUT_POLICY | BACKUP_POLICY | CHATBOT_POLICY | DECLARATIVE_POLICY_EC2 | RESOURCE_CONTROL_POLICY | SERVICE_CONTROL_POLICY | TAG_POLICY`
     */
    filter: string;
}
/**
 * A collection of values returned by getPolicies.
 */
export interface GetPoliciesResult {
    readonly filter: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of all the policy ids found.
     */
    readonly ids: string[];
}
/**
 * Data source for managing an AWS Organizations Policies.
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
 * const example = aws.organizations.getPolicies({
 *     filter: "SERVICE_CONTROL_POLICY",
 * });
 * const exampleGetPolicy = example.then(example => std.toset({
 *     input: example.ids,
 * })).then(invoke => .reduce((__obj, [__key, __value]) => ({ ...__obj, [__key]: aws.organizations.getPolicy({
 *     policyId: __value,
 * }) })));
 * ```
 */
export declare function getPoliciesOutput(args: GetPoliciesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPoliciesResult>;
/**
 * A collection of arguments for invoking getPolicies.
 */
export interface GetPoliciesOutputArgs {
    /**
     * The type of policies to be returned in the response. Valid values are `AISERVICES_OPT_OUT_POLICY | BACKUP_POLICY | CHATBOT_POLICY | DECLARATIVE_POLICY_EC2 | RESOURCE_CONTROL_POLICY | SERVICE_CONTROL_POLICY | TAG_POLICY`
     */
    filter: pulumi.Input<string>;
}
