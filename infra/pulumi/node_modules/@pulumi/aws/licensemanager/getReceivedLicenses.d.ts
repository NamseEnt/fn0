import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * This resource can be used to get a set of license ARNs matching a filter.
 *
 * ## Example Usage
 *
 * The following shows getting all license ARNs issued from the AWS marketplace. Providing no filter, would provide all license ARNs for the entire account.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.licensemanager.getReceivedLicenses({
 *     filters: [{
 *         name: "IssuerName",
 *         values: ["AWS/Marketplace"],
 *     }],
 * });
 * ```
 */
export declare function getReceivedLicenses(args?: GetReceivedLicensesArgs, opts?: pulumi.InvokeOptions): Promise<GetReceivedLicensesResult>;
/**
 * A collection of arguments for invoking getReceivedLicenses.
 */
export interface GetReceivedLicensesArgs {
    /**
     * Custom filter block as described below.
     */
    filters?: inputs.licensemanager.GetReceivedLicensesFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getReceivedLicenses.
 */
export interface GetReceivedLicensesResult {
    /**
     * List of all the license ARNs found.
     */
    readonly arns: string[];
    readonly filters?: outputs.licensemanager.GetReceivedLicensesFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * This resource can be used to get a set of license ARNs matching a filter.
 *
 * ## Example Usage
 *
 * The following shows getting all license ARNs issued from the AWS marketplace. Providing no filter, would provide all license ARNs for the entire account.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.licensemanager.getReceivedLicenses({
 *     filters: [{
 *         name: "IssuerName",
 *         values: ["AWS/Marketplace"],
 *     }],
 * });
 * ```
 */
export declare function getReceivedLicensesOutput(args?: GetReceivedLicensesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetReceivedLicensesResult>;
/**
 * A collection of arguments for invoking getReceivedLicenses.
 */
export interface GetReceivedLicensesOutputArgs {
    /**
     * Custom filter block as described below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.licensemanager.GetReceivedLicensesFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
