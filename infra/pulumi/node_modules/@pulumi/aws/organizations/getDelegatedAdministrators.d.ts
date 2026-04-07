import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get a list of AWS accounts that are designated as delegated administrators in this organization
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.organizations.getDelegatedAdministrators({
 *     servicePrincipal: "SERVICE PRINCIPAL",
 * });
 * ```
 */
export declare function getDelegatedAdministrators(args?: GetDelegatedAdministratorsArgs, opts?: pulumi.InvokeOptions): Promise<GetDelegatedAdministratorsResult>;
/**
 * A collection of arguments for invoking getDelegatedAdministrators.
 */
export interface GetDelegatedAdministratorsArgs {
    /**
     * Specifies a service principal name. If specified, then the operation lists the delegated administrators only for the specified service. If you don't specify a service principal, the operation lists all delegated administrators for all services in your organization.
     */
    servicePrincipal?: string;
}
/**
 * A collection of values returned by getDelegatedAdministrators.
 */
export interface GetDelegatedAdministratorsResult {
    /**
     * The list of delegated administrators in your organization, which have the following attributes:
     */
    readonly delegatedAdministrators: outputs.organizations.GetDelegatedAdministratorsDelegatedAdministrator[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly servicePrincipal?: string;
}
/**
 * Get a list of AWS accounts that are designated as delegated administrators in this organization
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.organizations.getDelegatedAdministrators({
 *     servicePrincipal: "SERVICE PRINCIPAL",
 * });
 * ```
 */
export declare function getDelegatedAdministratorsOutput(args?: GetDelegatedAdministratorsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDelegatedAdministratorsResult>;
/**
 * A collection of arguments for invoking getDelegatedAdministrators.
 */
export interface GetDelegatedAdministratorsOutputArgs {
    /**
     * Specifies a service principal name. If specified, then the operation lists the delegated administrators only for the specified service. If you don't specify a service principal, the operation lists all delegated administrators for all services in your organization.
     */
    servicePrincipal?: pulumi.Input<string>;
}
