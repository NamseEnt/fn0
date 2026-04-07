import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get all direct child accounts under a parent organizational unit. This provides all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const accounts = org.then(org => aws.organizations.getOrganizationalUnitDescendantAccounts({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitDescendantAccounts(args: GetOrganizationalUnitDescendantAccountsArgs, opts?: pulumi.InvokeOptions): Promise<GetOrganizationalUnitDescendantAccountsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnitDescendantAccounts.
 */
export interface GetOrganizationalUnitDescendantAccountsArgs {
    /**
     * The parent ID of the accounts.
     */
    parentId: string;
}
/**
 * A collection of values returned by getOrganizationalUnitDescendantAccounts.
 */
export interface GetOrganizationalUnitDescendantAccountsResult {
    /**
     * List of child accounts, which have the following attributes:
     */
    readonly accounts: outputs.organizations.GetOrganizationalUnitDescendantAccountsAccount[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly parentId: string;
}
/**
 * Get all direct child accounts under a parent organizational unit. This provides all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const accounts = org.then(org => aws.organizations.getOrganizationalUnitDescendantAccounts({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitDescendantAccountsOutput(args: GetOrganizationalUnitDescendantAccountsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOrganizationalUnitDescendantAccountsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnitDescendantAccounts.
 */
export interface GetOrganizationalUnitDescendantAccountsOutputArgs {
    /**
     * The parent ID of the accounts.
     */
    parentId: pulumi.Input<string>;
}
