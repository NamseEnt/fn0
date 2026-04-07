import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get all direct child accounts under a parent organizational unit. This only provides immediate children, not all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const accounts = org.then(org => aws.organizations.getOrganizationalUnitChildAccounts({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitChildAccounts(args: GetOrganizationalUnitChildAccountsArgs, opts?: pulumi.InvokeOptions): Promise<GetOrganizationalUnitChildAccountsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnitChildAccounts.
 */
export interface GetOrganizationalUnitChildAccountsArgs {
    /**
     * The parent ID of the accounts.
     */
    parentId: string;
}
/**
 * A collection of values returned by getOrganizationalUnitChildAccounts.
 */
export interface GetOrganizationalUnitChildAccountsResult {
    /**
     * List of child accounts, which have the following attributes:
     */
    readonly accounts: outputs.organizations.GetOrganizationalUnitChildAccountsAccount[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly parentId: string;
}
/**
 * Get all direct child accounts under a parent organizational unit. This only provides immediate children, not all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const accounts = org.then(org => aws.organizations.getOrganizationalUnitChildAccounts({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitChildAccountsOutput(args: GetOrganizationalUnitChildAccountsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOrganizationalUnitChildAccountsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnitChildAccounts.
 */
export interface GetOrganizationalUnitChildAccountsOutputArgs {
    /**
     * The parent ID of the accounts.
     */
    parentId: pulumi.Input<string>;
}
