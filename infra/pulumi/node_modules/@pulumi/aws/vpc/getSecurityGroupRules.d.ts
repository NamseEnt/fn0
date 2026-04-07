import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * This resource can be useful for getting back a set of security group rule IDs.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpc.getSecurityGroupRules({
 *     filters: [{
 *         name: "group-id",
 *         values: [securityGroupId],
 *     }],
 * });
 * ```
 */
export declare function getSecurityGroupRules(args?: GetSecurityGroupRulesArgs, opts?: pulumi.InvokeOptions): Promise<GetSecurityGroupRulesResult>;
/**
 * A collection of arguments for invoking getSecurityGroupRules.
 */
export interface GetSecurityGroupRulesArgs {
    /**
     * Custom filter block as described below.
     */
    filters?: inputs.vpc.GetSecurityGroupRulesFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired security group rule.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getSecurityGroupRules.
 */
export interface GetSecurityGroupRulesResult {
    readonly filters?: outputs.vpc.GetSecurityGroupRulesFilter[];
    readonly id: string;
    /**
     * List of all the security group rule IDs found.
     */
    readonly ids: string[];
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * This resource can be useful for getting back a set of security group rule IDs.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpc.getSecurityGroupRules({
 *     filters: [{
 *         name: "group-id",
 *         values: [securityGroupId],
 *     }],
 * });
 * ```
 */
export declare function getSecurityGroupRulesOutput(args?: GetSecurityGroupRulesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSecurityGroupRulesResult>;
/**
 * A collection of arguments for invoking getSecurityGroupRules.
 */
export interface GetSecurityGroupRulesOutputArgs {
    /**
     * Custom filter block as described below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.vpc.GetSecurityGroupRulesFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired security group rule.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
