import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for getting an AWS Organizations Organizational Unit.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const ou = org.then(org => aws.organizations.getOrganizationalUnit({
 *     parentId: org.roots?.[0]?.id,
 *     name: "dev",
 * }));
 * ```
 */
export declare function getOrganizationalUnit(args: GetOrganizationalUnitArgs, opts?: pulumi.InvokeOptions): Promise<GetOrganizationalUnitResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnit.
 */
export interface GetOrganizationalUnitArgs {
    /**
     * Name of the organizational unit
     */
    name: string;
    /**
     * Parent ID of the organizational unit.
     */
    parentId: string;
}
/**
 * A collection of values returned by getOrganizationalUnit.
 */
export interface GetOrganizationalUnitResult {
    /**
     * ARN of the organizational unit
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly parentId: string;
}
/**
 * Data source for getting an AWS Organizations Organizational Unit.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const org = aws.organizations.getOrganization({});
 * const ou = org.then(org => aws.organizations.getOrganizationalUnit({
 *     parentId: org.roots?.[0]?.id,
 *     name: "dev",
 * }));
 * ```
 */
export declare function getOrganizationalUnitOutput(args: GetOrganizationalUnitOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetOrganizationalUnitResult>;
/**
 * A collection of arguments for invoking getOrganizationalUnit.
 */
export interface GetOrganizationalUnitOutputArgs {
    /**
     * Name of the organizational unit
     */
    name: pulumi.Input<string>;
    /**
     * Parent ID of the organizational unit.
     */
    parentId: pulumi.Input<string>;
}
