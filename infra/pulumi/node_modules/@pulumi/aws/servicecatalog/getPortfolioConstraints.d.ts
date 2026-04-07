import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides information on Service Catalog Portfolio Constraints.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getPortfolioConstraints({
 *     portfolioId: "port-3lli3b3an",
 * });
 * ```
 */
export declare function getPortfolioConstraints(args: GetPortfolioConstraintsArgs, opts?: pulumi.InvokeOptions): Promise<GetPortfolioConstraintsResult>;
/**
 * A collection of arguments for invoking getPortfolioConstraints.
 */
export interface GetPortfolioConstraintsArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: string;
    /**
     * Portfolio identifier.
     *
     * The following arguments are optional:
     */
    portfolioId: string;
    /**
     * Product identifier.
     */
    productId?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPortfolioConstraints.
 */
export interface GetPortfolioConstraintsResult {
    readonly acceptLanguage?: string;
    /**
     * List of information about the constraints. See details below.
     */
    readonly details: outputs.servicecatalog.GetPortfolioConstraintsDetail[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Identifier of the portfolio the product resides in. The constraint applies only to the instance of the product that lives within this portfolio.
     */
    readonly portfolioId: string;
    /**
     * Identifier of the product the constraint applies to. A constraint applies to a specific instance of a product within a certain portfolio.
     */
    readonly productId?: string;
    readonly region: string;
}
/**
 * Provides information on Service Catalog Portfolio Constraints.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getPortfolioConstraints({
 *     portfolioId: "port-3lli3b3an",
 * });
 * ```
 */
export declare function getPortfolioConstraintsOutput(args: GetPortfolioConstraintsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPortfolioConstraintsResult>;
/**
 * A collection of arguments for invoking getPortfolioConstraints.
 */
export interface GetPortfolioConstraintsOutputArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     *
     * The following arguments are optional:
     */
    portfolioId: pulumi.Input<string>;
    /**
     * Product identifier.
     */
    productId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
