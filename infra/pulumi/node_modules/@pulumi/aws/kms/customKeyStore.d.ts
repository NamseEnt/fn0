import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS KMS (Key Management) Custom Key Store.
 *
 * ## Example Usage
 *
 * ### CloudHSM
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const test = new aws.kms.CustomKeyStore("test", {
 *     cloudHsmClusterId: cloudHsmClusterId,
 *     customKeyStoreName: "kms-custom-key-store-test",
 *     keyStorePassword: "noplaintextpasswords1",
 *     trustAnchorCertificate: std.file({
 *         input: "anchor-certificate.crt",
 *     }).then(invoke => invoke.result),
 * });
 * ```
 *
 * ### External Key Store (VPC)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.CustomKeyStore("example", {
 *     customKeyStoreName: "example-vpc-xks",
 *     customKeyStoreType: "EXTERNAL_KEY_STORE",
 *     xksProxyAuthenticationCredential: {
 *         accessKeyId: ephemeralAccessKeyId,
 *         rawSecretAccessKey: ephemeralSecretAccessKey,
 *     },
 *     xksProxyConnectivity: "VPC_ENDPOINT_SERVICE",
 *     xksProxyUriEndpoint: "https://myproxy-private.xks.example.com",
 *     xksProxyUriPath: "/kms/xks/v1",
 *     xksProxyVpcEndpointServiceName: "com.amazonaws.vpce.us-east-1.vpce-svc-example",
 * });
 * ```
 *
 * ### External Key Store (Public)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.CustomKeyStore("example", {
 *     customKeyStoreName: "example-public-xks",
 *     customKeyStoreType: "EXTERNAL_KEY_STORE",
 *     xksProxyAuthenticationCredential: {
 *         accessKeyId: ephemeralAccessKeyId,
 *         rawSecretAccessKey: ephemeralSecretAccessKey,
 *     },
 *     xksProxyConnectivity: "PUBLIC_ENDPOINT",
 *     xksProxyUriEndpoint: "https://myproxy.xks.example.com",
 *     xksProxyUriPath: "/kms/xks/v1",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import KMS (Key Management) Custom Key Store using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:kms/customKeyStore:CustomKeyStore example cks-5ebd4ef395a96288e
 * ```
 */
export declare class CustomKeyStore extends pulumi.CustomResource {
    /**
     * Get an existing CustomKeyStore resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomKeyStoreState, opts?: pulumi.CustomResourceOptions): CustomKeyStore;
    /**
     * Returns true if the given object is an instance of CustomKeyStore.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomKeyStore;
    readonly cloudHsmClusterId: pulumi.Output<string | undefined>;
    /**
     * Unique name for Custom Key Store.
     *
     * The following arguments are optional:
     */
    readonly customKeyStoreName: pulumi.Output<string>;
    /**
     * Specifies the type of key store to create. Valid values are `AWS_CLOUDHSM` and `EXTERNAL_KEY_STORE`. If omitted, AWS will default the value to `AWS_CLOUDHSM`.
     */
    readonly customKeyStoreType: pulumi.Output<string>;
    readonly keyStorePassword: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly trustAnchorCertificate: pulumi.Output<string | undefined>;
    readonly xksProxyAuthenticationCredential: pulumi.Output<outputs.kms.CustomKeyStoreXksProxyAuthenticationCredential | undefined>;
    readonly xksProxyConnectivity: pulumi.Output<string | undefined>;
    readonly xksProxyUriEndpoint: pulumi.Output<string | undefined>;
    readonly xksProxyUriPath: pulumi.Output<string | undefined>;
    readonly xksProxyVpcEndpointServiceName: pulumi.Output<string | undefined>;
    /**
     * Create a CustomKeyStore resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomKeyStoreArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomKeyStore resources.
 */
export interface CustomKeyStoreState {
    cloudHsmClusterId?: pulumi.Input<string>;
    /**
     * Unique name for Custom Key Store.
     *
     * The following arguments are optional:
     */
    customKeyStoreName?: pulumi.Input<string>;
    /**
     * Specifies the type of key store to create. Valid values are `AWS_CLOUDHSM` and `EXTERNAL_KEY_STORE`. If omitted, AWS will default the value to `AWS_CLOUDHSM`.
     */
    customKeyStoreType?: pulumi.Input<string>;
    keyStorePassword?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    trustAnchorCertificate?: pulumi.Input<string>;
    xksProxyAuthenticationCredential?: pulumi.Input<inputs.kms.CustomKeyStoreXksProxyAuthenticationCredential>;
    xksProxyConnectivity?: pulumi.Input<string>;
    xksProxyUriEndpoint?: pulumi.Input<string>;
    xksProxyUriPath?: pulumi.Input<string>;
    xksProxyVpcEndpointServiceName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CustomKeyStore resource.
 */
export interface CustomKeyStoreArgs {
    cloudHsmClusterId?: pulumi.Input<string>;
    /**
     * Unique name for Custom Key Store.
     *
     * The following arguments are optional:
     */
    customKeyStoreName: pulumi.Input<string>;
    /**
     * Specifies the type of key store to create. Valid values are `AWS_CLOUDHSM` and `EXTERNAL_KEY_STORE`. If omitted, AWS will default the value to `AWS_CLOUDHSM`.
     */
    customKeyStoreType?: pulumi.Input<string>;
    keyStorePassword?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    trustAnchorCertificate?: pulumi.Input<string>;
    xksProxyAuthenticationCredential?: pulumi.Input<inputs.kms.CustomKeyStoreXksProxyAuthenticationCredential>;
    xksProxyConnectivity?: pulumi.Input<string>;
    xksProxyUriEndpoint?: pulumi.Input<string>;
    xksProxyUriPath?: pulumi.Input<string>;
    xksProxyVpcEndpointServiceName?: pulumi.Input<string>;
}
