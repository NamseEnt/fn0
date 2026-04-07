import * as pulumi from "@pulumi/pulumi";
/**
 * Accepts a License Manager grant. This allows for sharing licenses with other aws accounts.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.licensemanager.LicenseGrantAccepter("test", {grantArn: "arn:aws:license-manager::123456789012:grant:g-1cf9fba4ba2f42dcab11c686c4b4d329"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_licensemanager_grant_accepter` using the grant arn. For example:
 *
 * ```sh
 * $ pulumi import aws:licensemanager/licenseGrantAccepter:LicenseGrantAccepter test arn:aws:license-manager::123456789012:grant:g-1cf9fba4ba2f42dcab11c686c4b4d329
 * ```
 */
export declare class LicenseGrantAccepter extends pulumi.CustomResource {
    /**
     * Get an existing LicenseGrantAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LicenseGrantAccepterState, opts?: pulumi.CustomResourceOptions): LicenseGrantAccepter;
    /**
     * Returns true if the given object is an instance of LicenseGrantAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LicenseGrantAccepter;
    /**
     * A list of the allowed operations for the grant.
     */
    readonly allowedOperations: pulumi.Output<string[]>;
    /**
     * The ARN of the grant to accept.
     */
    readonly grantArn: pulumi.Output<string>;
    /**
     * The home region for the license.
     */
    readonly homeRegion: pulumi.Output<string>;
    /**
     * The ARN of the license for the grant.
     */
    readonly licenseArn: pulumi.Output<string>;
    /**
     * The Name of the grant.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The parent ARN.
     */
    readonly parentArn: pulumi.Output<string>;
    /**
     * The target account for the grant.
     */
    readonly principal: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The grant status.
     */
    readonly status: pulumi.Output<string>;
    /**
     * The grant version.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a LicenseGrantAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LicenseGrantAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LicenseGrantAccepter resources.
 */
export interface LicenseGrantAccepterState {
    /**
     * A list of the allowed operations for the grant.
     */
    allowedOperations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the grant to accept.
     */
    grantArn?: pulumi.Input<string>;
    /**
     * The home region for the license.
     */
    homeRegion?: pulumi.Input<string>;
    /**
     * The ARN of the license for the grant.
     */
    licenseArn?: pulumi.Input<string>;
    /**
     * The Name of the grant.
     */
    name?: pulumi.Input<string>;
    /**
     * The parent ARN.
     */
    parentArn?: pulumi.Input<string>;
    /**
     * The target account for the grant.
     */
    principal?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The grant status.
     */
    status?: pulumi.Input<string>;
    /**
     * The grant version.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LicenseGrantAccepter resource.
 */
export interface LicenseGrantAccepterArgs {
    /**
     * The ARN of the grant to accept.
     */
    grantArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
