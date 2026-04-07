import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon Managed Grafana workspace license association resource.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assume = new aws.iam.Role("assume", {
 *     name: "grafana-assume",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "grafana.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const exampleWorkspace = new aws.grafana.Workspace("example", {
 *     accountAccessType: "CURRENT_ACCOUNT",
 *     authenticationProviders: ["SAML"],
 *     permissionType: "SERVICE_MANAGED",
 *     roleArn: assume.arn,
 * });
 * const example = new aws.grafana.LicenseAssociation("example", {
 *     licenseType: "ENTERPRISE_FREE_TRIAL",
 *     workspaceId: exampleWorkspace.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Grafana workspace license association using the workspace's `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:grafana/licenseAssociation:LicenseAssociation example g-2054c75a02
 * ```
 */
export declare class LicenseAssociation extends pulumi.CustomResource {
    /**
     * Get an existing LicenseAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LicenseAssociationState, opts?: pulumi.CustomResourceOptions): LicenseAssociation;
    /**
     * Returns true if the given object is an instance of LicenseAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LicenseAssociation;
    /**
     * If `licenseType` is set to `ENTERPRISE_FREE_TRIAL`, this is the expiration date of the free trial.
     */
    readonly freeTrialExpiration: pulumi.Output<string>;
    /**
     * A token from Grafana Labs that ties your AWS account with a Grafana Labs account.
     */
    readonly grafanaToken: pulumi.Output<string | undefined>;
    /**
     * If `licenseType` is set to `ENTERPRISE`, this is the expiration date of the enterprise license.
     */
    readonly licenseExpiration: pulumi.Output<string>;
    /**
     * The type of license for the workspace license association. Valid values are `ENTERPRISE` and `ENTERPRISE_FREE_TRIAL`.
     */
    readonly licenseType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The workspace id.
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a LicenseAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LicenseAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LicenseAssociation resources.
 */
export interface LicenseAssociationState {
    /**
     * If `licenseType` is set to `ENTERPRISE_FREE_TRIAL`, this is the expiration date of the free trial.
     */
    freeTrialExpiration?: pulumi.Input<string>;
    /**
     * A token from Grafana Labs that ties your AWS account with a Grafana Labs account.
     */
    grafanaToken?: pulumi.Input<string>;
    /**
     * If `licenseType` is set to `ENTERPRISE`, this is the expiration date of the enterprise license.
     */
    licenseExpiration?: pulumi.Input<string>;
    /**
     * The type of license for the workspace license association. Valid values are `ENTERPRISE` and `ENTERPRISE_FREE_TRIAL`.
     */
    licenseType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The workspace id.
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LicenseAssociation resource.
 */
export interface LicenseAssociationArgs {
    /**
     * A token from Grafana Labs that ties your AWS account with a Grafana Labs account.
     */
    grafanaToken?: pulumi.Input<string>;
    /**
     * The type of license for the workspace license association. Valid values are `ENTERPRISE` and `ENTERPRISE_FREE_TRIAL`.
     */
    licenseType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The workspace id.
     */
    workspaceId: pulumi.Input<string>;
}
