import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a License Manager grant. This allows for sharing licenses with other AWS accounts.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.licensemanager.LicenseGrant("test", {
 *     name: "share-license-with-account",
 *     allowedOperations: [
 *         "ListPurchasedLicenses",
 *         "CheckoutLicense",
 *         "CheckInLicense",
 *         "ExtendConsumptionLicense",
 *         "CreateToken",
 *     ],
 *     licenseArn: "arn:aws:license-manager::111111111111:license:l-exampleARN",
 *     principal: "arn:aws:iam::111111111112:root",
 *     homeRegion: "us-east-1",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_licensemanager_grant` using the grant arn. For example:
 *
 * ```sh
 * $ pulumi import aws:licensemanager/licenseGrant:LicenseGrant test arn:aws:license-manager::123456789011:grant:g-01d313393d9e443d8664cc054db1e089
 * ```
 */
export declare class LicenseGrant extends pulumi.CustomResource {
    /**
     * Get an existing LicenseGrant resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LicenseGrantState, opts?: pulumi.CustomResourceOptions): LicenseGrant;
    /**
     * Returns true if the given object is an instance of LicenseGrant.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LicenseGrant;
    /**
     * A list of the allowed operations for the grant. This is a subset of the allowed operations on the license.
     */
    readonly allowedOperations: pulumi.Output<string[]>;
    /**
     * The grant ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The home region for the license.
     */
    readonly homeRegion: pulumi.Output<string>;
    /**
     * The ARN of the license to grant.
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
     * The target account for the grant in the form of the ARN for an account principal of the root user.
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
     * Create a LicenseGrant resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LicenseGrantArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LicenseGrant resources.
 */
export interface LicenseGrantState {
    /**
     * A list of the allowed operations for the grant. This is a subset of the allowed operations on the license.
     */
    allowedOperations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The grant ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The home region for the license.
     */
    homeRegion?: pulumi.Input<string>;
    /**
     * The ARN of the license to grant.
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
     * The target account for the grant in the form of the ARN for an account principal of the root user.
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
 * The set of arguments for constructing a LicenseGrant resource.
 */
export interface LicenseGrantArgs {
    /**
     * A list of the allowed operations for the grant. This is a subset of the allowed operations on the license.
     */
    allowedOperations: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the license to grant.
     */
    licenseArn: pulumi.Input<string>;
    /**
     * The Name of the grant.
     */
    name?: pulumi.Input<string>;
    /**
     * The target account for the grant in the form of the ARN for an account principal of the root user.
     */
    principal: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
