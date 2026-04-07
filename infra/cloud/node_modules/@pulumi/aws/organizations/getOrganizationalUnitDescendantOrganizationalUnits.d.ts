import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get all direct child organizational units under a parent organizational unit. This provides all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const ous = org.then(org => aws.organizations.getOrganizationalUnitDescendantOrganizationalUnits({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitDescendantOrganizationalUnits(args: GetOrganizationalUnitDescendantOrganizationalUnitsArgs, opts?: pulumi.InvokeOptions): Promise<GetOrganizationalUnitDescendantOrganizationalUnitsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnitDescendantOrganizationalUnits.
 */
export interface GetOrganizationalUnitDescendantOrganizationalUnitsArgs {
    /**
     * Parent ID of the organizational unit.
     */
    parentId: string;
}
/**
 * A collection of values returned by getOrganizationalUnitDescendantOrganizationalUnits.
 */
export interface GetOrganizationalUnitDescendantOrganizationalUnitsResult {
    /**
     * List of child organizational units, which have the following attributes:
     */
    readonly childrens: outputs.organizations.GetOrganizationalUnitDescendantOrganizationalUnitsChildren[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly parentId: string;
}
/**
 * Get all direct child organizational units under a parent organizational unit. This provides all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const ous = org.then(org => aws.organizations.getOrganizationalUnitDescendantOrganizationalUnits({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitDescendantOrganizationalUnitsOutput(args: GetOrganizationalUnitDescendantOrganizationalUnitsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOrganizationalUnitDescendantOrganizationalUnitsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnitDescendantOrganizationalUnits.
 */
export interface GetOrganizationalUnitDescendantOrganizationalUnitsOutputArgs {
    /**
     * Parent ID of the organizational unit.
     */
    parentId: pulumi.Input<string>;
}
