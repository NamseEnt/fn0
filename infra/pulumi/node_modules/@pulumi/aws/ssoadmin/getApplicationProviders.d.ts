import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS SSO Admin Application Providers.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getApplicationProviders({});
 * ```
 */
export declare function getApplicationProviders(args?: GetApplicationProvidersArgs, opts?: pulumi.InvokeOptions): Promise<GetApplicationProvidersResult>;
/**
 * A collection of arguments for invoking getApplicationProviders.
 */
export interface GetApplicationProvidersArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getApplicationProviders.
 */
export interface GetApplicationProvidersResult {
    /**
     * A list of application providers available in the current region. See `applicationProviders` below.
     */
    readonly applicationProviders: outputs.ssoadmin.GetApplicationProvidersApplicationProvider[];
    /**
     * AWS region.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing AWS SSO Admin Application Providers.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getApplicationProviders({});
 * ```
 */
export declare function getApplicationProvidersOutput(args?: GetApplicationProvidersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetApplicationProvidersResult>;
/**
 * A collection of arguments for invoking getApplicationProviders.
 */
export interface GetApplicationProvidersOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
