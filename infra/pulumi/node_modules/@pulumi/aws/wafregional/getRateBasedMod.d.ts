import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.wafregional.RateBasedRule` Retrieves a WAF Regional Rate Based Rule Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.wafregional.getRateBasedMod({
 *     name: "tfWAFRegionalRateBasedRule",
 * });
 * ```
 */
export declare function getRateBasedMod(args: GetRateBasedModArgs, opts?: pulumi.InvokeOptions): Promise<GetRateBasedModResult>;
/**
 * A collection of arguments for invoking getRateBasedMod.
 */
export interface GetRateBasedModArgs {
    /**
     * Name of the WAF Regional rate based rule.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getRateBasedMod.
 */
export interface GetRateBasedModResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
}
/**
 * `aws.wafregional.RateBasedRule` Retrieves a WAF Regional Rate Based Rule Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.wafregional.getRateBasedMod({
 *     name: "tfWAFRegionalRateBasedRule",
 * });
 * ```
 */
export declare function getRateBasedModOutput(args: GetRateBasedModOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRateBasedModResult>;
/**
 * A collection of arguments for invoking getRateBasedMod.
 */
export interface GetRateBasedModOutputArgs {
    /**
     * Name of the WAF Regional rate based rule.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
