import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Verified Permissions Policy Store.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.verifiedpermissions.getPolicyStore({
 *     id: "example",
 * });
 * ```
 */
export declare function getPolicyStore(args: GetPolicyStoreArgs, opts?: pulumi.InvokeOptions): Promise<GetPolicyStoreResult>;
/**
 * A collection of arguments for invoking getPolicyStore.
 */
export interface GetPolicyStoreArgs {
    /**
     * The ID of the Policy Store.
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPolicyStore.
 */
export interface GetPolicyStoreResult {
    /**
     * The ARN of the Policy Store.
     */
    readonly arn: string;
    /**
     * The date the Policy Store was created.
     */
    readonly createdDate: string;
    /**
     * Whether the policy store can be deleted.
     */
    readonly deletionProtection: string;
    readonly description: string;
    readonly id: string;
    /**
     * The date the Policy Store was last updated.
     */
    readonly lastUpdatedDate: string;
    readonly region: string;
    /**
     * Map of key-value pairs associated with the policy store.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Validation settings for the policy store.
     */
    readonly validationSettings: outputs.verifiedpermissions.GetPolicyStoreValidationSetting[];
}
/**
 * Data source for managing an AWS Verified Permissions Policy Store.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.verifiedpermissions.getPolicyStore({
 *     id: "example",
 * });
 * ```
 */
export declare function getPolicyStoreOutput(args: GetPolicyStoreOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPolicyStoreResult>;
/**
 * A collection of arguments for invoking getPolicyStore.
 */
export interface GetPolicyStoreOutputArgs {
    /**
     * The ID of the Policy Store.
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
