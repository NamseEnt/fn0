import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get all direct child organizational units under a parent organizational unit. This only provides immediate children, not all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const ou = org.then(org => aws.organizations.getOrganizationalUnits({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnits(args: GetOrganizationalUnitsArgs, opts?: pulumi.InvokeOptions): Promise<GetOrganizationalUnitsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnits.
 */
export interface GetOrganizationalUnitsArgs {
    /**
     * Parent ID of the organizational unit.
     */
    parentId: string;
}
/**
 * A collection of values returned by getOrganizationalUnits.
 */
export interface GetOrganizationalUnitsResult {
    /**
     * List of child organizational units, which have the following attributes:
     */
    readonly children: outputs.organizations.GetOrganizationalUnitsChild[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly parentId: string;
}
/**
 * Get all direct child organizational units under a parent organizational unit. This only provides immediate children, not all children.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const ou = org.then(org => aws.organizations.getOrganizationalUnits({
 *     parentId: org.roots?.[0]?.id,
 * }));
 * ```
 */
export declare function getOrganizationalUnitsOutput(args: GetOrganizationalUnitsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOrganizationalUnitsResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnits.
 */
export interface GetOrganizationalUnitsOutputArgs {
    /**
     * Parent ID of the organizational unit.
     */
    parentId: pulumi.Input<string>;
}
