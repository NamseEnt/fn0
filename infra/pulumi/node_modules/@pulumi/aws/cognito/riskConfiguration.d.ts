import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Cognito Risk Configuration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.RiskConfiguration("example", {
 *     userPoolId: exampleAwsCognitoUserPool.id,
 *     riskExceptionConfiguration: {
 *         blockedIpRangeLists: ["10.10.10.10/32"],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Import using the user pool ID and Client ID separated by a `:`:
 *
 * __Using `pulumi import` to import__ Cognito Risk Configurations using the user pool ID or the user pool ID and Client Id separated by a `:`. For example:
 *
 * Import using the user pool ID:
 *
 * ```sh
 * $ pulumi import aws:cognito/riskConfiguration:RiskConfiguration main example
 * ```
 * Import using the user pool ID and Client ID separated by a `:`:
 *
 * ```sh
 * $ pulumi import aws:cognito/riskConfiguration:RiskConfiguration main example:example
 * ```
 */
export declare class RiskConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing RiskConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RiskConfigurationState, opts?: pulumi.CustomResourceOptions): RiskConfiguration;
    /**
     * Returns true if the given object is an instance of RiskConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RiskConfiguration;
    /**
     * The account takeover risk configuration. See details below.
     */
    readonly accountTakeoverRiskConfiguration: pulumi.Output<outputs.cognito.RiskConfigurationAccountTakeoverRiskConfiguration | undefined>;
    /**
     * The app client ID. When the client ID is not provided, the same risk configuration is applied to all the clients in the User Pool.
     */
    readonly clientId: pulumi.Output<string | undefined>;
    /**
     * The compromised credentials risk configuration. See details below.
     */
    readonly compromisedCredentialsRiskConfiguration: pulumi.Output<outputs.cognito.RiskConfigurationCompromisedCredentialsRiskConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The configuration to override the risk decision. See details below.
     */
    readonly riskExceptionConfiguration: pulumi.Output<outputs.cognito.RiskConfigurationRiskExceptionConfiguration | undefined>;
    /**
     * The user pool ID.
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * Create a RiskConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RiskConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RiskConfiguration resources.
 */
export interface RiskConfigurationState {
    /**
     * The account takeover risk configuration. See details below.
     */
    accountTakeoverRiskConfiguration?: pulumi.Input<inputs.cognito.RiskConfigurationAccountTakeoverRiskConfiguration>;
    /**
     * The app client ID. When the client ID is not provided, the same risk configuration is applied to all the clients in the User Pool.
     */
    clientId?: pulumi.Input<string>;
    /**
     * The compromised credentials risk configuration. See details below.
     */
    compromisedCredentialsRiskConfiguration?: pulumi.Input<inputs.cognito.RiskConfigurationCompromisedCredentialsRiskConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration to override the risk decision. See details below.
     */
    riskExceptionConfiguration?: pulumi.Input<inputs.cognito.RiskConfigurationRiskExceptionConfiguration>;
    /**
     * The user pool ID.
     */
    userPoolId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RiskConfiguration resource.
 */
export interface RiskConfigurationArgs {
    /**
     * The account takeover risk configuration. See details below.
     */
    accountTakeoverRiskConfiguration?: pulumi.Input<inputs.cognito.RiskConfigurationAccountTakeoverRiskConfiguration>;
    /**
     * The app client ID. When the client ID is not provided, the same risk configuration is applied to all the clients in the User Pool.
     */
    clientId?: pulumi.Input<string>;
    /**
     * The compromised credentials risk configuration. See details below.
     */
    compromisedCredentialsRiskConfiguration?: pulumi.Input<inputs.cognito.RiskConfigurationCompromisedCredentialsRiskConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The configuration to override the risk decision. See details below.
     */
    riskExceptionConfiguration?: pulumi.Input<inputs.cognito.RiskConfigurationRiskExceptionConfiguration>;
    /**
     * The user pool ID.
     */
    userPoolId: pulumi.Input<string>;
}
