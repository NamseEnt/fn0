import * as pulumi from "@pulumi/pulumi";
/**
 * Provides information for a Service Catalog Portfolio.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const portfolio = aws.servicecatalog.getPortfolio({
 *     id: "port-07052002",
 * });
 * ```
 */
export declare function getPortfolio(args: GetPortfolioArgs, opts?: pulumi.InvokeOptions): Promise<GetPortfolioResult>;
/**
 * A collection of arguments for invoking getPortfolio.
 */
export interface GetPortfolioArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: string;
    /**
     * Portfolio identifier.
     *
     * The following arguments are optional:
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Tags applied to the portfolio.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getPortfolio.
 */
export interface GetPortfolioResult {
    readonly acceptLanguage?: string;
    /**
     * Portfolio ARN.
     */
    readonly arn: string;
    /**
     * Time the portfolio was created.
     */
    readonly createdTime: string;
    /**
     * Description of the portfolio
     */
    readonly description: string;
    readonly id: string;
    /**
     * Portfolio name.
     */
    readonly name: string;
    /**
     * Name of the person or organization who owns the portfolio.
     */
    readonly providerName: string;
    readonly region: string;
    /**
     * Tags applied to the portfolio.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides information for a Service Catalog Portfolio.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const portfolio = aws.servicecatalog.getPortfolio({
 *     id: "port-07052002",
 * });
 * ```
 */
export declare function getPortfolioOutput(args: GetPortfolioOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPortfolioResult>;
/**
 * A collection of arguments for invoking getPortfolio.
 */
export interface GetPortfolioOutputArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     *
     * The following arguments are optional:
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags applied to the portfolio.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
