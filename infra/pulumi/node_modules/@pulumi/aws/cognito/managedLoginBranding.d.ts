import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages branding settings for a user pool style and associates it with an app client.
 *
 * ## Example Usage
 *
 * ### Default Branding Style
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const client = new aws.cognito.ManagedLoginBranding("client", {
 *     clientId: example.id,
 *     userPoolId: exampleAwsCognitoUserPool.id,
 *     useCognitoProvidedValues: true,
 * });
 * ```
 *
 * ### Custom Branding Style
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const client = new aws.cognito.ManagedLoginBranding("client", {
 *     clientId: example.id,
 *     userPoolId: exampleAwsCognitoUserPool.id,
 *     assets: [{
 *         bytes: std.filebase64({
 *             input: "login_branding_asset.svg",
 *         }).then(invoke => invoke.result),
 *         category: "PAGE_HEADER_BACKGROUND",
 *         colorMode: "DARK",
 *         extension: "SVG",
 *     }],
 *     settings: JSON.stringify({}),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cognito branding settings using `user_pool_id` and `managed_login_branding_id` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/managedLoginBranding:ManagedLoginBranding example us-west-2_rSss9Zltr,06c6ae7b-1e66-46d2-87a9-1203ea3307bd
 * ```
 */
export declare class ManagedLoginBranding extends pulumi.CustomResource {
    /**
     * Get an existing ManagedLoginBranding resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ManagedLoginBrandingState, opts?: pulumi.CustomResourceOptions): ManagedLoginBranding;
    /**
     * Returns true if the given object is an instance of ManagedLoginBranding.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ManagedLoginBranding;
    /**
     * Image files to apply to roles like backgrounds, logos, and icons. See details below.
     */
    readonly assets: pulumi.Output<outputs.cognito.ManagedLoginBrandingAsset[] | undefined>;
    /**
     * App client that the branding style is for.
     */
    readonly clientId: pulumi.Output<string>;
    /**
     * ID of the managed login branding style.
     */
    readonly managedLoginBrandingId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * JSON document with the the settings to apply to the style.
     */
    readonly settings: pulumi.Output<string | undefined>;
    /**
     * Settings including Amazon Cognito defaults.
     */
    readonly settingsAll: pulumi.Output<string>;
    /**
     * When `true`, applies the default branding style options.
     */
    readonly useCognitoProvidedValues: pulumi.Output<boolean>;
    /**
     * User pool the client belongs to.
     *
     * The following arguments are optional:
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * Create a ManagedLoginBranding resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ManagedLoginBrandingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ManagedLoginBranding resources.
 */
export interface ManagedLoginBrandingState {
    /**
     * Image files to apply to roles like backgrounds, logos, and icons. See details below.
     */
    assets?: pulumi.Input<pulumi.Input<inputs.cognito.ManagedLoginBrandingAsset>[]>;
    /**
     * App client that the branding style is for.
     */
    clientId?: pulumi.Input<string>;
    /**
     * ID of the managed login branding style.
     */
    managedLoginBrandingId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * JSON document with the the settings to apply to the style.
     */
    settings?: pulumi.Input<string>;
    /**
     * Settings including Amazon Cognito defaults.
     */
    settingsAll?: pulumi.Input<string>;
    /**
     * When `true`, applies the default branding style options.
     */
    useCognitoProvidedValues?: pulumi.Input<boolean>;
    /**
     * User pool the client belongs to.
     *
     * The following arguments are optional:
     */
    userPoolId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ManagedLoginBranding resource.
 */
export interface ManagedLoginBrandingArgs {
    /**
     * Image files to apply to roles like backgrounds, logos, and icons. See details below.
     */
    assets?: pulumi.Input<pulumi.Input<inputs.cognito.ManagedLoginBrandingAsset>[]>;
    /**
     * App client that the branding style is for.
     */
    clientId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * JSON document with the the settings to apply to the style.
     */
    settings?: pulumi.Input<string>;
    /**
     * When `true`, applies the default branding style options.
     */
    useCognitoProvidedValues?: pulumi.Input<boolean>;
    /**
     * User pool the client belongs to.
     *
     * The following arguments are optional:
     */
    userPoolId: pulumi.Input<string>;
}
