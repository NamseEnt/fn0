import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an IAM Security Token Service Preferences resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.SecurityTokenServicePreferences("example", {globalEndpointTokenVersion: "v2Token"});
 * ```
 */
export declare class SecurityTokenServicePreferences extends pulumi.CustomResource {
    /**
     * Get an existing SecurityTokenServicePreferences resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecurityTokenServicePreferencesState, opts?: pulumi.CustomResourceOptions): SecurityTokenServicePreferences;
    /**
     * Returns true if the given object is an instance of SecurityTokenServicePreferences.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecurityTokenServicePreferences;
    /**
     * The version of the STS global endpoint token. Valid values: `v1Token`, `v2Token`.
     */
    readonly globalEndpointTokenVersion: pulumi.Output<string>;
    /**
     * Create a SecurityTokenServicePreferences resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecurityTokenServicePreferencesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecurityTokenServicePreferences resources.
 */
export interface SecurityTokenServicePreferencesState {
    /**
     * The version of the STS global endpoint token. Valid values: `v1Token`, `v2Token`.
     */
    globalEndpointTokenVersion?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SecurityTokenServicePreferences resource.
 */
export interface SecurityTokenServicePreferencesArgs {
    /**
     * The version of the STS global endpoint token. Valid values: `v1Token`, `v2Token`.
     */
    globalEndpointTokenVersion: pulumi.Input<string>;
}
