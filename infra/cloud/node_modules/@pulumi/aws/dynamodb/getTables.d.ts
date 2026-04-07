import * as pulumi from "@pulumi/pulumi";
/**
 * Returns a list of all AWS DynamoDB table names in a region.
 *
 * ## Example Usage
 *
 * The following example retrieves a list of all DynamoDB table names in a region.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.dynamodb.getTables({});
 * export const tableNames = all.then(all => all.names);
 * ```
 */
export declare function getTables(args?: GetTablesArgs, opts?: pulumi.InvokeOptions): Promise<GetTablesResult>;
/**
 * A collection of arguments for invoking getTables.
 */
export interface GetTablesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getTables.
 */
export interface GetTablesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * A list of all the DynamoDB table names found.
     */
    readonly names: string[];
    readonly region: string;
}
/**
 * Returns a list of all AWS DynamoDB table names in a region.
 *
 * ## Example Usage
 *
 * The following example retrieves a list of all DynamoDB table names in a region.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.dynamodb.getTables({});
 * export const tableNames = all.then(all => all.names);
 * ```
 */
export declare function getTablesOutput(args?: GetTablesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetTablesResult>;
/**
 * A collection of arguments for invoking getTables.
 */
export interface GetTablesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
