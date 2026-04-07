import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get a list the AWS services for which the specified account is a delegated administrator
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.organizations.getDelegatedServices({
 *     accountId: "AWS ACCOUNT ID",
 * });
 * ```
 */
export declare function getDelegatedServices(args: GetDelegatedServicesArgs, opts?: pulumi.InvokeOptions): Promise<GetDelegatedServicesResult>;
/**
 * A collection of arguments for invoking getDelegatedServices.
 */
export interface GetDelegatedServicesArgs {
    /**
     * Account ID number of a delegated administrator account in the organization.
     */
    accountId: string;
}
/**
 * A collection of values returned by getDelegatedServices.
 */
export interface GetDelegatedServicesResult {
    readonly accountId: string;
    /**
     * Services for which the account is a delegated administrator, which have the following attributes:
     */
    readonly delegatedServices: outputs.organizations.GetDelegatedServicesDelegatedService[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
}
/**
 * Get a list the AWS services for which the specified account is a delegated administrator
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.organizations.getDelegatedServices({
 *     accountId: "AWS ACCOUNT ID",
 * });
 * ```
 */
export declare function getDelegatedServicesOutput(args: GetDelegatedServicesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDelegatedServicesResult>;
/**
 * A collection of arguments for invoking getDelegatedServices.
 */
export interface GetDelegatedServicesOutputArgs {
    /**
     * Account ID number of a delegated administrator account in the organization.
     */
    accountId: pulumi.Input<string>;
}
