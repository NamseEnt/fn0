import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Verified Permissions Identity Source.
 *
 * ## Example Usage
 *
 * ### Cognito User Pool Configuration Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedpermissions.PolicyStore("example", {validationSettings: {
 *     mode: "STRICT",
 * }});
 * const exampleUserPool = new aws.cognito.UserPool("example", {name: "example"});
 * const exampleUserPoolClient = new aws.cognito.UserPoolClient("example", {
 *     name: "example",
 *     userPoolId: exampleUserPool.id,
 *     explicitAuthFlows: ["ADMIN_NO_SRP_AUTH"],
 * });
 * const exampleIdentitySource = new aws.verifiedpermissions.IdentitySource("example", {
 *     policyStoreId: example.id,
 *     configuration: {
 *         cognitoUserPoolConfiguration: {
 *             userPoolArn: exampleUserPool.arn,
 *             clientIds: [exampleUserPoolClient.id],
 *         },
 *     },
 * });
 * ```
 *
 * ### OpenID Connect Configuration Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedpermissions.PolicyStore("example", {validationSettings: {
 *     mode: "STRICT",
 * }});
 * const exampleIdentitySource = new aws.verifiedpermissions.IdentitySource("example", {
 *     policyStoreId: example.id,
 *     configuration: {
 *         openIdConnectConfiguration: {
 *             issuer: "https://auth.example.com",
 *             tokenSelection: {
 *                 accessTokenOnly: {
 *                     audiences: ["https://myapp.example.com"],
 *                     principalIdClaim: "sub",
 *                 },
 *             },
 *             entityIdPrefix: "MyOIDCProvider",
 *             groupConfiguration: {
 *                 groupClaim: "groups",
 *                 groupEntityType: "MyCorp::UserGroup",
 *             },
 *         },
 *     },
 *     principalEntityType: "MyCorp::User",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Permissions Identity Source using the `policy_store_id:identity_source_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:verifiedpermissions/identitySource:IdentitySource example policy-store-id-12345678:identity-source-id-12345678
 * ```
 */
export declare class IdentitySource extends pulumi.CustomResource {
    /**
     * Get an existing IdentitySource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IdentitySourceState, opts?: pulumi.CustomResourceOptions): IdentitySource;
    /**
     * Returns true if the given object is an instance of IdentitySource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IdentitySource;
    /**
     * Specifies the details required to communicate with the identity provider (IdP) associated with this identity source. See Configuration below.
     */
    readonly configuration: pulumi.Output<outputs.verifiedpermissions.IdentitySourceConfiguration | undefined>;
    /**
     * Specifies the ID of the policy store in which you want to store this identity source.
     */
    readonly policyStoreId: pulumi.Output<string>;
    /**
     * Specifies the namespace and data type of the principals generated for identities authenticated by the new identity source.
     */
    readonly principalEntityType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a IdentitySource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IdentitySourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IdentitySource resources.
 */
export interface IdentitySourceState {
    /**
     * Specifies the details required to communicate with the identity provider (IdP) associated with this identity source. See Configuration below.
     */
    configuration?: pulumi.Input<inputs.verifiedpermissions.IdentitySourceConfiguration>;
    /**
     * Specifies the ID of the policy store in which you want to store this identity source.
     */
    policyStoreId?: pulumi.Input<string>;
    /**
     * Specifies the namespace and data type of the principals generated for identities authenticated by the new identity source.
     */
    principalEntityType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a IdentitySource resource.
 */
export interface IdentitySourceArgs {
    /**
     * Specifies the details required to communicate with the identity provider (IdP) associated with this identity source. See Configuration below.
     */
    configuration?: pulumi.Input<inputs.verifiedpermissions.IdentitySourceConfiguration>;
    /**
     * Specifies the ID of the policy store in which you want to store this identity source.
     */
    policyStoreId: pulumi.Input<string>;
    /**
     * Specifies the namespace and data type of the principals generated for identities authenticated by the new identity source.
     */
    principalEntityType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
