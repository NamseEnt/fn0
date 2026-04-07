import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Transfer Family Web App Customization.
 *
 * ## Example Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer Family Web App Customization using the `web_app_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/webAppCustomization:WebAppCustomization example webapp-12345678901234567890
 * ```
 */
export declare class WebAppCustomization extends pulumi.CustomResource {
    /**
     * Get an existing WebAppCustomization resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WebAppCustomizationState, opts?: pulumi.CustomResourceOptions): WebAppCustomization;
    /**
     * Returns true if the given object is an instance of WebAppCustomization.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WebAppCustomization;
    readonly faviconFile: pulumi.Output<string>;
    readonly logoFile: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Title of the web app. Must be between 1 and 100 characters in length (an empty string is not allowed). To remove the title, omit this argument from your configuration.
     */
    readonly title: pulumi.Output<string | undefined>;
    /**
     * The identifier of the web app to be customized.
     *
     * The following arguments are optional:
     */
    readonly webAppId: pulumi.Output<string>;
    /**
     * Create a WebAppCustomization resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WebAppCustomizationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WebAppCustomization resources.
 */
export interface WebAppCustomizationState {
    faviconFile?: pulumi.Input<string>;
    logoFile?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Title of the web app. Must be between 1 and 100 characters in length (an empty string is not allowed). To remove the title, omit this argument from your configuration.
     */
    title?: pulumi.Input<string>;
    /**
     * The identifier of the web app to be customized.
     *
     * The following arguments are optional:
     */
    webAppId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WebAppCustomization resource.
 */
export interface WebAppCustomizationArgs {
    faviconFile?: pulumi.Input<string>;
    logoFile?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Title of the web app. Must be between 1 and 100 characters in length (an empty string is not allowed). To remove the title, omit this argument from your configuration.
     */
    title?: pulumi.Input<string>;
    /**
     * The identifier of the web app to be customized.
     *
     * The following arguments are optional:
     */
    webAppId: pulumi.Input<string>;
}
