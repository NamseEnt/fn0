import * as pulumi from "@pulumi/pulumi";
/**
 * Provides information on a Service Catalog Constraint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getConstraint({
 *     acceptLanguage: "en",
 *     id: "cons-hrvy0335",
 * });
 * ```
 */
export declare function getConstraint(args: GetConstraintArgs, opts?: pulumi.InvokeOptions): Promise<GetConstraintResult>;
/**
 * A collection of arguments for invoking getConstraint.
 */
export interface GetConstraintArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: string;
    /**
     * Description of the constraint.
     */
    description?: string;
    /**
     * Constraint identifier.
     *
     * The following arguments are optional:
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getConstraint.
 */
export interface GetConstraintResult {
    readonly acceptLanguage?: string;
    /**
     * Description of the constraint.
     */
    readonly description: string;
    readonly id: string;
    /**
     * Owner of the constraint.
     */
    readonly owner: string;
    /**
     * Constraint parameters in JSON format.
     */
    readonly parameters: string;
    /**
     * Portfolio identifier.
     */
    readonly portfolioId: string;
    /**
     * Product identifier.
     */
    readonly productId: string;
    readonly region: string;
    /**
     * Constraint status.
     */
    readonly status: string;
    /**
     * Type of constraint. Valid values are `LAUNCH`, `NOTIFICATION`, `RESOURCE_UPDATE`, `STACKSET`, and `TEMPLATE`.
     */
    readonly type: string;
}
/**
 * Provides information on a Service Catalog Constraint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getConstraint({
 *     acceptLanguage: "en",
 *     id: "cons-hrvy0335",
 * });
 * ```
 */
export declare function getConstraintOutput(args: GetConstraintOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetConstraintResult>;
/**
 * A collection of arguments for invoking getConstraint.
 */
export interface GetConstraintOutputArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Description of the constraint.
     */
    description?: pulumi.Input<string>;
    /**
     * Constraint identifier.
     *
     * The following arguments are optional:
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
