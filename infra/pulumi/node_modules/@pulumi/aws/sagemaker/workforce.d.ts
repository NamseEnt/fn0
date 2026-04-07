import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Workforce resource.
 *
 * ## Example Usage
 *
 * ### Cognito Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleUserPool = new aws.cognito.UserPool("example", {name: "example"});
 * const exampleUserPoolClient = new aws.cognito.UserPoolClient("example", {
 *     name: "example",
 *     generateSecret: true,
 *     userPoolId: exampleUserPool.id,
 * });
 * const exampleUserPoolDomain = new aws.cognito.UserPoolDomain("example", {
 *     domain: "example",
 *     userPoolId: exampleUserPool.id,
 * });
 * const example = new aws.sagemaker.Workforce("example", {
 *     workforceName: "example",
 *     cognitoConfig: {
 *         clientId: exampleUserPoolClient.id,
 *         userPool: exampleUserPoolDomain.userPoolId,
 *     },
 * });
 * ```
 *
 * ### Oidc Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Workforce("example", {
 *     workforceName: "example",
 *     oidcConfig: {
 *         authorizationEndpoint: "https://example.com",
 *         clientId: "example",
 *         clientSecret: "example",
 *         issuer: "https://example.com",
 *         jwksUri: "https://example.com",
 *         logoutEndpoint: "https://example.com",
 *         tokenEndpoint: "https://example.com",
 *         userInfoEndpoint: "https://example.com",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Workforces using the `workforce_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/workforce:Workforce example example
 * ```
 */
export declare class Workforce extends pulumi.CustomResource {
    /**
     * Get an existing Workforce resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkforceState, opts?: pulumi.CustomResourceOptions): Workforce;
    /**
     * Returns true if the given object is an instance of Workforce.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Workforce;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Workforce.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Use this parameter to configure an Amazon Cognito private workforce. A single Cognito workforce is created using and corresponds to a single Amazon Cognito user pool. Conflicts with `oidcConfig`. see Cognito Config details below.
     */
    readonly cognitoConfig: pulumi.Output<outputs.sagemaker.WorkforceCognitoConfig | undefined>;
    /**
     * Use this parameter to configure a private workforce using your own OIDC Identity Provider. Conflicts with `cognitoConfig`. see OIDC Config details below.
     */
    readonly oidcConfig: pulumi.Output<outputs.sagemaker.WorkforceOidcConfig | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of IP address ranges Used to create an allow list of IP addresses for a private workforce. By default, a workforce isn't restricted to specific IP addresses. see Source Ip Config details below.
     */
    readonly sourceIpConfig: pulumi.Output<outputs.sagemaker.WorkforceSourceIpConfig>;
    /**
     * The subdomain for your OIDC Identity Provider.
     */
    readonly subdomain: pulumi.Output<string>;
    /**
     * The name of the Workforce (must be unique).
     */
    readonly workforceName: pulumi.Output<string>;
    /**
     * configure a workforce using VPC. see Workforce VPC Config details below.
     */
    readonly workforceVpcConfig: pulumi.Output<outputs.sagemaker.WorkforceWorkforceVpcConfig | undefined>;
    /**
     * Create a Workforce resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkforceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Workforce resources.
 */
export interface WorkforceState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Workforce.
     */
    arn?: pulumi.Input<string>;
    /**
     * Use this parameter to configure an Amazon Cognito private workforce. A single Cognito workforce is created using and corresponds to a single Amazon Cognito user pool. Conflicts with `oidcConfig`. see Cognito Config details below.
     */
    cognitoConfig?: pulumi.Input<inputs.sagemaker.WorkforceCognitoConfig>;
    /**
     * Use this parameter to configure a private workforce using your own OIDC Identity Provider. Conflicts with `cognitoConfig`. see OIDC Config details below.
     */
    oidcConfig?: pulumi.Input<inputs.sagemaker.WorkforceOidcConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of IP address ranges Used to create an allow list of IP addresses for a private workforce. By default, a workforce isn't restricted to specific IP addresses. see Source Ip Config details below.
     */
    sourceIpConfig?: pulumi.Input<inputs.sagemaker.WorkforceSourceIpConfig>;
    /**
     * The subdomain for your OIDC Identity Provider.
     */
    subdomain?: pulumi.Input<string>;
    /**
     * The name of the Workforce (must be unique).
     */
    workforceName?: pulumi.Input<string>;
    /**
     * configure a workforce using VPC. see Workforce VPC Config details below.
     */
    workforceVpcConfig?: pulumi.Input<inputs.sagemaker.WorkforceWorkforceVpcConfig>;
}
/**
 * The set of arguments for constructing a Workforce resource.
 */
export interface WorkforceArgs {
    /**
     * Use this parameter to configure an Amazon Cognito private workforce. A single Cognito workforce is created using and corresponds to a single Amazon Cognito user pool. Conflicts with `oidcConfig`. see Cognito Config details below.
     */
    cognitoConfig?: pulumi.Input<inputs.sagemaker.WorkforceCognitoConfig>;
    /**
     * Use this parameter to configure a private workforce using your own OIDC Identity Provider. Conflicts with `cognitoConfig`. see OIDC Config details below.
     */
    oidcConfig?: pulumi.Input<inputs.sagemaker.WorkforceOidcConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of IP address ranges Used to create an allow list of IP addresses for a private workforce. By default, a workforce isn't restricted to specific IP addresses. see Source Ip Config details below.
     */
    sourceIpConfig?: pulumi.Input<inputs.sagemaker.WorkforceSourceIpConfig>;
    /**
     * The name of the Workforce (must be unique).
     */
    workforceName: pulumi.Input<string>;
    /**
     * configure a workforce using VPC. see Workforce VPC Config details below.
     */
    workforceVpcConfig?: pulumi.Input<inputs.sagemaker.WorkforceWorkforceVpcConfig>;
}
