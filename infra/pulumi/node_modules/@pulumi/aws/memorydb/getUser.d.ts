import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides information about a MemoryDB User.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.memorydb.getUser({
 *     userName: "my-user",
 * });
 * ```
 */
export declare function getUser(args: GetUserArgs, opts?: pulumi.InvokeOptions): Promise<GetUserResult>;
/**
 * A collection of arguments for invoking getUser.
 */
export interface GetUserArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the user.
     */
    tags?: {
        [key: string]: string;
    };
    /**
     * Name of the user.
     */
    userName: string;
}
/**
 * A collection of values returned by getUser.
 */
export interface GetUserResult {
    /**
     * Access permissions string used for this user.
     */
    readonly accessString: string;
    /**
     * ARN of the user.
     */
    readonly arn: string;
    /**
     * Denotes the user's authentication properties.
     */
    readonly authenticationModes: outputs.memorydb.GetUserAuthenticationMode[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Minimum engine version supported for the user.
     */
    readonly minimumEngineVersion: string;
    readonly region: string;
    /**
     * Map of tags assigned to the user.
     */
    readonly tags: {
        [key: string]: string;
    };
    readonly userName: string;
}
/**
 * Provides information about a MemoryDB User.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.memorydb.getUser({
 *     userName: "my-user",
 * });
 * ```
 */
export declare function getUserOutput(args: GetUserOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUserResult>;
/**
 * A collection of arguments for invoking getUser.
 */
export interface GetUserOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the user.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name of the user.
     */
    userName: pulumi.Input<string>;
}
