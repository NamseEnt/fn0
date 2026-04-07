import * as pulumi from "@pulumi/pulumi";
/**
 * Provides information about a MemoryDB ACL.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.memorydb.getAcl({
 *     name: "my-acl",
 * });
 * ```
 */
export declare function getAcl(args: GetAclArgs, opts?: pulumi.InvokeOptions): Promise<GetAclResult>;
/**
 * A collection of arguments for invoking getAcl.
 */
export interface GetAclArgs {
    /**
     * Name of the ACL.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the ACL.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getAcl.
 */
export interface GetAclResult {
    /**
     * ARN of the ACL.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The minimum engine version supported by the ACL.
     */
    readonly minimumEngineVersion: string;
    readonly name: string;
    readonly region: string;
    /**
     * Map of tags assigned to the ACL.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Set of MemoryDB user names included in this ACL.
     */
    readonly userNames: string[];
}
/**
 * Provides information about a MemoryDB ACL.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.memorydb.getAcl({
 *     name: "my-acl",
 * });
 * ```
 */
export declare function getAclOutput(args: GetAclOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAclResult>;
/**
 * A collection of arguments for invoking getAcl.
 */
export interface GetAclOutputArgs {
    /**
     * Name of the ACL.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the ACL.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
