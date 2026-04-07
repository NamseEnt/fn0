import * as pulumi from "@pulumi/pulumi";
/**
 * Manages Service Catalog AWS Organizations Access, a portfolio sharing feature through AWS Organizations. This allows Service Catalog to receive updates on your organization in order to sync your shares with the current structure. This resource will prompt AWS to set `organizations:EnableAWSServiceAccess` on your behalf so that your shares can be in sync with any changes in your AWS Organizations structure.
 *
 * > **NOTE:** This resource can only be used by the management account in the organization. In other words, a delegated administrator is not authorized to use the resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.OrganizationsAccess("example", {enabled: true});
 * ```
 */
export declare class OrganizationsAccess extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationsAccess resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationsAccessState, opts?: pulumi.CustomResourceOptions): OrganizationsAccess;
    /**
     * Returns true if the given object is an instance of OrganizationsAccess.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationsAccess;
    /**
     * Whether to enable AWS Organizations access.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Create a OrganizationsAccess resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationsAccessArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationsAccess resources.
 */
export interface OrganizationsAccessState {
    /**
     * Whether to enable AWS Organizations access.
     */
    enabled?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a OrganizationsAccess resource.
 */
export interface OrganizationsAccessArgs {
    /**
     * Whether to enable AWS Organizations access.
     */
    enabled: pulumi.Input<boolean>;
}
