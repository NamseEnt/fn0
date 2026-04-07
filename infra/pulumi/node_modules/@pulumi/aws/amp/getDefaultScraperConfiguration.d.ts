import * as pulumi from "@pulumi/pulumi";
/**
 * Returns the default scraper configuration used when Amazon EKS creates a scraper for you.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.amp.getDefaultScraperConfiguration({});
 * ```
 */
export declare function getDefaultScraperConfiguration(args?: GetDefaultScraperConfigurationArgs, opts?: pulumi.InvokeOptions): Promise<GetDefaultScraperConfigurationResult>;
/**
 * A collection of arguments for invoking getDefaultScraperConfiguration.
 */
export interface GetDefaultScraperConfigurationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDefaultScraperConfiguration.
 */
export interface GetDefaultScraperConfigurationResult {
    /**
     * The configuration file.
     */
    readonly configuration: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Returns the default scraper configuration used when Amazon EKS creates a scraper for you.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.amp.getDefaultScraperConfiguration({});
 * ```
 */
export declare function getDefaultScraperConfigurationOutput(args?: GetDefaultScraperConfigurationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDefaultScraperConfigurationResult>;
/**
 * A collection of arguments for invoking getDefaultScraperConfiguration.
 */
export interface GetDefaultScraperConfigurationOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
