import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS CloudWatch Observability Access Manager Links.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.oam.getLinks({});
 * ```
 */
export declare function getLinks(args?: GetLinksArgs, opts?: pulumi.InvokeOptions): Promise<GetLinksResult>;
/**
 * A collection of arguments for invoking getLinks.
 */
export interface GetLinksArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getLinks.
 */
export interface GetLinksResult {
    /**
     * Set of ARN of the Links.
     */
    readonly arns: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS CloudWatch Observability Access Manager Links.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.oam.getLinks({});
 * ```
 */
export declare function getLinksOutput(args?: GetLinksOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLinksResult>;
/**
 * A collection of arguments for invoking getLinks.
 */
export interface GetLinksOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
