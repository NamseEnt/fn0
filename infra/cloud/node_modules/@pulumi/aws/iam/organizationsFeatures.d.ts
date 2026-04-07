import * as pulumi from "@pulumi/pulumi";
/**
 * Manages centralized root access features across AWS member accounts managed using AWS Organizations. More information about managing root access in IAM can be found in the [Centralize root access for member accounts](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_root-enable-root-access.html).
 *
 * > **NOTE:** The AWS account utilizing this resource must be an Organizations management account. Also, you must enable trusted access for AWS Identity and Access Management in AWS Organizations.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.organizations.Organization("example", {
 *     awsServiceAccessPrincipals: ["iam.amazonaws.com"],
 *     featureSet: "ALL",
 * });
 * const exampleOrganizationsFeatures = new aws.iam.OrganizationsFeatures("example", {enabledFeatures: [
 *     "RootCredentialsManagement",
 *     "RootSessions",
 * ]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import root access features using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/organizationsFeatures:OrganizationsFeatures example o-1234567
 * ```
 */
export declare class OrganizationsFeatures extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationsFeatures resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationsFeaturesState, opts?: pulumi.CustomResourceOptions): OrganizationsFeatures;
    /**
     * Returns true if the given object is an instance of OrganizationsFeatures.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationsFeatures;
    /**
     * List of IAM features to enable. Valid values are `RootCredentialsManagement` and `RootSessions`.
     */
    readonly enabledFeatures: pulumi.Output<string[]>;
    /**
     * Create a OrganizationsFeatures resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationsFeaturesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationsFeatures resources.
 */
export interface OrganizationsFeaturesState {
    /**
     * List of IAM features to enable. Valid values are `RootCredentialsManagement` and `RootSessions`.
     */
    enabledFeatures?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a OrganizationsFeatures resource.
 */
export interface OrganizationsFeaturesArgs {
    /**
     * List of IAM features to enable. Valid values are `RootCredentialsManagement` and `RootSessions`.
     */
    enabledFeatures: pulumi.Input<pulumi.Input<string>[]>;
}
