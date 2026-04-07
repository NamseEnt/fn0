import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to fetch information about a Quantum Ledger Database.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.qldb.getLedger({
 *     name: "an_example_ledger",
 * });
 * ```
 */
export declare function getLedger(args: GetLedgerArgs, opts?: pulumi.InvokeOptions): Promise<GetLedgerResult>;
/**
 * A collection of arguments for invoking getLedger.
 */
export interface GetLedgerArgs {
    /**
     * Friendly name of the ledger to match.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getLedger.
 */
export interface GetLedgerResult {
    readonly arn: string;
    readonly deletionProtection: boolean;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly kmsKey: string;
    readonly name: string;
    readonly permissionsMode: string;
    readonly region: string;
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Use this data source to fetch information about a Quantum Ledger Database.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.qldb.getLedger({
 *     name: "an_example_ledger",
 * });
 * ```
 */
export declare function getLedgerOutput(args: GetLedgerOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLedgerResult>;
/**
 * A collection of arguments for invoking getLedger.
 */
export interface GetLedgerOutputArgs {
    /**
     * Friendly name of the ledger to match.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
