import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS Redshift Data Shares.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshift.getDataShares({});
 * ```
 */
export declare function getDataShares(args?: GetDataSharesArgs, opts?: pulumi.InvokeOptions): Promise<GetDataSharesResult>;
/**
 * A collection of arguments for invoking getDataShares.
 */
export interface GetDataSharesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDataShares.
 */
export interface GetDataSharesResult {
    /**
     * An array of all data shares in the current region. See `dataShares` below.
     */
    readonly dataShares: outputs.redshift.GetDataSharesDataShare[];
    /**
     * AWS region.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing AWS Redshift Data Shares.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshift.getDataShares({});
 * ```
 */
export declare function getDataSharesOutput(args?: GetDataSharesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDataSharesResult>;
/**
 * A collection of arguments for invoking getDataShares.
 */
export interface GetDataSharesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
