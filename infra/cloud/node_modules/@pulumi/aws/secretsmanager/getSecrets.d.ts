import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Use this data source to get the ARNs and names of Secrets Manager secrets matching the specified criteria.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.secretsmanager.getSecrets({
 *     filters: [{
 *         name: "name",
 *         values: ["example"],
 *     }],
 * });
 * ```
 */
export declare function getSecrets(args?: GetSecretsArgs, opts?: pulumi.InvokeOptions): Promise<GetSecretsResult>;
/**
 * A collection of arguments for invoking getSecrets.
 */
export interface GetSecretsArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: inputs.secretsmanager.GetSecretsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSecrets.
 */
export interface GetSecretsResult {
    /**
     * Set of ARNs of the matched Secrets Manager secrets.
     */
    readonly arns: string[];
    readonly filters?: outputs.secretsmanager.GetSecretsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of names of the matched Secrets Manager secrets.
     */
    readonly names: string[];
    readonly region: string;
}
/**
 * Use this data source to get the ARNs and names of Secrets Manager secrets matching the specified criteria.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.secretsmanager.getSecrets({
 *     filters: [{
 *         name: "name",
 *         values: ["example"],
 *     }],
 * });
 * ```
 */
export declare function getSecretsOutput(args?: GetSecretsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSecretsResult>;
/**
 * A collection of arguments for invoking getSecrets.
 */
export interface GetSecretsOutputArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.secretsmanager.GetSecretsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
