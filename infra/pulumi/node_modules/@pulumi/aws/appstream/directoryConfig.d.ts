import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AppStream Directory Config.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appstream.DirectoryConfig("example", {
 *     directoryName: "NAME OF DIRECTORY",
 *     organizationalUnitDistinguishedNames: ["DISTINGUISHED NAME"],
 *     serviceAccountCredentials: {
 *         accountName: "NAME OF ACCOUNT",
 *         accountPassword: "PASSWORD OF ACCOUNT",
 *     },
 *     certificateBasedAuthProperties: {
 *         certificateAuthorityArn: "ARN OF CERTIFICATE AUTHORITY",
 *         status: "STATUS OF CERTIFICATE BASED AUTHENTICATION",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appstream_directory_config` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:appstream/directoryConfig:DirectoryConfig example directoryNameExample
 * ```
 */
export declare class DirectoryConfig extends pulumi.CustomResource {
    /**
     * Get an existing DirectoryConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DirectoryConfigState, opts?: pulumi.CustomResourceOptions): DirectoryConfig;
    /**
     * Returns true if the given object is an instance of DirectoryConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DirectoryConfig;
    /**
     * Configuration block for the certificate-based authentication properties used to authenticate SAML 2.0 Identity Provider (IdP) user identities to Active Directory domain-joined streaming instances. See `certificateBasedAuthProperties` below.
     */
    readonly certificateBasedAuthProperties: pulumi.Output<outputs.appstream.DirectoryConfigCertificateBasedAuthProperties | undefined>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the directory config was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Fully qualified name of the directory.
     */
    readonly directoryName: pulumi.Output<string>;
    /**
     * Distinguished names of the organizational units for computer accounts.
     */
    readonly organizationalUnitDistinguishedNames: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for the name of the directory and organizational unit (OU) to use to join the directory config to a Microsoft Active Directory domain. See `serviceAccountCredentials` below.
     */
    readonly serviceAccountCredentials: pulumi.Output<outputs.appstream.DirectoryConfigServiceAccountCredentials>;
    /**
     * Create a DirectoryConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DirectoryConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DirectoryConfig resources.
 */
export interface DirectoryConfigState {
    /**
     * Configuration block for the certificate-based authentication properties used to authenticate SAML 2.0 Identity Provider (IdP) user identities to Active Directory domain-joined streaming instances. See `certificateBasedAuthProperties` below.
     */
    certificateBasedAuthProperties?: pulumi.Input<inputs.appstream.DirectoryConfigCertificateBasedAuthProperties>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the directory config was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Fully qualified name of the directory.
     */
    directoryName?: pulumi.Input<string>;
    /**
     * Distinguished names of the organizational units for computer accounts.
     */
    organizationalUnitDistinguishedNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the name of the directory and organizational unit (OU) to use to join the directory config to a Microsoft Active Directory domain. See `serviceAccountCredentials` below.
     */
    serviceAccountCredentials?: pulumi.Input<inputs.appstream.DirectoryConfigServiceAccountCredentials>;
}
/**
 * The set of arguments for constructing a DirectoryConfig resource.
 */
export interface DirectoryConfigArgs {
    /**
     * Configuration block for the certificate-based authentication properties used to authenticate SAML 2.0 Identity Provider (IdP) user identities to Active Directory domain-joined streaming instances. See `certificateBasedAuthProperties` below.
     */
    certificateBasedAuthProperties?: pulumi.Input<inputs.appstream.DirectoryConfigCertificateBasedAuthProperties>;
    /**
     * Fully qualified name of the directory.
     */
    directoryName: pulumi.Input<string>;
    /**
     * Distinguished names of the organizational units for computer accounts.
     */
    organizationalUnitDistinguishedNames: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the name of the directory and organizational unit (OU) to use to join the directory config to a Microsoft Active Directory domain. See `serviceAccountCredentials` below.
     */
    serviceAccountCredentials: pulumi.Input<inputs.appstream.DirectoryConfigServiceAccountCredentials>;
}
