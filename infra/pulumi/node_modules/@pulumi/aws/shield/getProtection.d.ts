import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Shield Protection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.shield.getProtection({
 *     protectionId: "abc123",
 * });
 * ```
 *
 * ### By Resource ARN
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.shield.getProtection({
 *     resourceArn: "arn:aws:globalaccelerator::123456789012:accelerator/1234abcd-abcd-1234-abcd-1234abcdefgh",
 * });
 * ```
 */
export declare function getProtection(args?: GetProtectionArgs, opts?: pulumi.InvokeOptions): Promise<GetProtectionResult>;
/**
 * A collection of arguments for invoking getProtection.
 */
export interface GetProtectionArgs {
    /**
     * Unique identifier for the protection.
     */
    protectionId?: string;
    /**
     * ARN (Amazon Resource Name) of the resource being protected.
     *
     * > Exactly one of `protectionId` or `resourceArn` is required.
     */
    resourceArn?: string;
}
/**
 * A collection of values returned by getProtection.
 */
export interface GetProtectionResult {
    readonly id: string;
    /**
     * Name of the protection.
     */
    readonly name: string;
    /**
     * ARN of the protection.
     */
    readonly protectionArn: string;
    readonly protectionId: string;
    readonly resourceArn: string;
}
/**
 * Data source for managing an AWS Shield Protection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.shield.getProtection({
 *     protectionId: "abc123",
 * });
 * ```
 *
 * ### By Resource ARN
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.shield.getProtection({
 *     resourceArn: "arn:aws:globalaccelerator::123456789012:accelerator/1234abcd-abcd-1234-abcd-1234abcdefgh",
 * });
 * ```
 */
export declare function getProtectionOutput(args?: GetProtectionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetProtectionResult>;
/**
 * A collection of arguments for invoking getProtection.
 */
export interface GetProtectionOutputArgs {
    /**
     * Unique identifier for the protection.
     */
    protectionId?: pulumi.Input<string>;
    /**
     * ARN (Amazon Resource Name) of the resource being protected.
     *
     * > Exactly one of `protectionId` or `resourceArn` is required.
     */
    resourceArn?: pulumi.Input<string>;
}
