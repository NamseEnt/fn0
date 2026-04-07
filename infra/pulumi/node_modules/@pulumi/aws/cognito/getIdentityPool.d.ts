import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Cognito Identity Pool.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cognito.getIdentityPool({
 *     identityPoolName: "test pool",
 * });
 * ```
 */
export declare function getIdentityPool(args: GetIdentityPoolArgs, opts?: pulumi.InvokeOptions): Promise<GetIdentityPoolResult>;
/**
 * A collection of arguments for invoking getIdentityPool.
 */
export interface GetIdentityPoolArgs {
    /**
     * The Cognito Identity Pool name.
     */
    identityPoolName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * A map of tags to assigned to the Identity Pool.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getIdentityPool.
 */
export interface GetIdentityPoolResult {
    /**
     * Whether the classic / basic authentication flow is enabled.
     */
    readonly allowClassicFlow: boolean;
    /**
     * Whether the identity pool supports unauthenticated logins or not.
     */
    readonly allowUnauthenticatedIdentities: boolean;
    /**
     * ARN of the Pool.
     */
    readonly arn: string;
    /**
     * An array of Amazon Cognito Identity user pools and their client IDs.
     */
    readonly cognitoIdentityProviders: outputs.cognito.GetIdentityPoolCognitoIdentityProvider[];
    /**
     * The "domain" by which Cognito will refer to your users.
     */
    readonly developerProviderName: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly identityPoolName: string;
    /**
     * Set of OpendID Connect provider ARNs.
     */
    readonly openidConnectProviderArns: string[];
    readonly region: string;
    /**
     * An array of Amazon Resource Names (ARNs) of the SAML provider for your identity.
     */
    readonly samlProviderArns: string[];
    /**
     * Key-Value pairs mapping provider names to provider app IDs.
     */
    readonly supportedLoginProviders: {
        [key: string]: string;
    };
    /**
     * A map of tags to assigned to the Identity Pool.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS Cognito Identity Pool.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cognito.getIdentityPool({
 *     identityPoolName: "test pool",
 * });
 * ```
 */
export declare function getIdentityPoolOutput(args: GetIdentityPoolOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetIdentityPoolResult>;
/**
 * A collection of arguments for invoking getIdentityPool.
 */
export interface GetIdentityPoolOutputArgs {
    /**
     * The Cognito Identity Pool name.
     */
    identityPoolName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assigned to the Identity Pool.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
