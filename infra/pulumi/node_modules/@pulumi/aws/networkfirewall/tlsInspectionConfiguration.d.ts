import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Network Firewall TLS Inspection Configuration.
 *
 * ## Example Usage
 *
 * > **NOTE:** You must configure either inbound inspection, outbound inspection, or both.
 *
 * ### Basic inbound/ingress inspection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.TlsInspectionConfiguration("example", {
 *     name: "example",
 *     description: "example",
 *     encryptionConfigurations: [{
 *         keyId: "AWS_OWNED_KMS_KEY",
 *         type: "AWS_OWNED_KMS_KEY",
 *     }],
 *     tlsInspectionConfiguration: {
 *         serverCertificateConfiguration: {
 *             serverCertificates: [{
 *                 resourceArn: example1.arn,
 *             }],
 *             scopes: [{
 *                 protocols: [6],
 *                 destinationPorts: [{
 *                     fromPort: 443,
 *                     toPort: 443,
 *                 }],
 *                 destinations: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *                 sourcePorts: [{
 *                     fromPort: 0,
 *                     toPort: 65535,
 *                 }],
 *                 sources: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### Basic outbound/engress inspection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.TlsInspectionConfiguration("example", {
 *     name: "example",
 *     description: "example",
 *     encryptionConfigurations: [{
 *         keyId: "AWS_OWNED_KMS_KEY",
 *         type: "AWS_OWNED_KMS_KEY",
 *     }],
 *     tlsInspectionConfiguration: {
 *         serverCertificateConfiguration: {
 *             certificateAuthorityArn: example1.arn,
 *             checkCertificateRevocationStatus: {
 *                 revokedStatusAction: "REJECT",
 *                 unknownStatusAction: "PASS",
 *             },
 *             scopes: [{
 *                 protocols: [6],
 *                 destinationPorts: [{
 *                     fromPort: 443,
 *                     toPort: 443,
 *                 }],
 *                 destinations: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *                 sourcePorts: [{
 *                     fromPort: 0,
 *                     toPort: 65535,
 *                 }],
 *                 sources: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### Inbound with encryption configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "example",
 *     deletionWindowInDays: 7,
 * });
 * const exampleTlsInspectionConfiguration = new aws.networkfirewall.TlsInspectionConfiguration("example", {
 *     name: "example",
 *     description: "example",
 *     encryptionConfigurations: [{
 *         keyId: example.arn,
 *         type: "CUSTOMER_KMS",
 *     }],
 *     tlsInspectionConfiguration: {
 *         serverCertificateConfiguration: {
 *             serverCertificates: [{
 *                 resourceArn: example1.arn,
 *             }],
 *             scopes: [{
 *                 protocols: [6],
 *                 destinationPorts: [{
 *                     fromPort: 443,
 *                     toPort: 443,
 *                 }],
 *                 destinations: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *                 sourcePorts: [{
 *                     fromPort: 0,
 *                     toPort: 65535,
 *                 }],
 *                 sources: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### Outbound with encryption configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "example",
 *     deletionWindowInDays: 7,
 * });
 * const exampleTlsInspectionConfiguration = new aws.networkfirewall.TlsInspectionConfiguration("example", {
 *     name: "example",
 *     description: "example",
 *     encryptionConfigurations: [{
 *         keyId: example.arn,
 *         type: "CUSTOMER_KMS",
 *     }],
 *     tlsInspectionConfiguration: {
 *         serverCertificateConfigurations: [{
 *             certificateAuthorityArn: example1.arn,
 *             checkCertificateRevocationStatus: [{
 *                 revokedStatusAction: "REJECT",
 *                 unknownStatusAction: "PASS",
 *             }],
 *             scope: [{
 *                 protocols: [6],
 *                 destinationPorts: [{
 *                     fromPort: 443,
 *                     toPort: 443,
 *                 }],
 *                 destination: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *                 sourcePorts: [{
 *                     fromPort: 0,
 *                     toPort: 65535,
 *                 }],
 *                 source: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *             }],
 *         }],
 *     },
 * });
 * ```
 *
 * ### Combined inbound and outbound
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.TlsInspectionConfiguration("example", {
 *     name: "example",
 *     description: "example",
 *     encryptionConfigurations: [{
 *         keyId: "AWS_OWNED_KMS_KEY",
 *         type: "AWS_OWNED_KMS_KEY",
 *     }],
 *     tlsInspectionConfiguration: {
 *         serverCertificateConfiguration: {
 *             certificateAuthorityArn: example1.arn,
 *             checkCertificateRevocationStatus: {
 *                 revokedStatusAction: "REJECT",
 *                 unknownStatusAction: "PASS",
 *             },
 *             serverCertificates: [{
 *                 resourceArn: example2.arn,
 *             }],
 *             scopes: [{
 *                 protocols: [6],
 *                 destinationPorts: [{
 *                     fromPort: 443,
 *                     toPort: 443,
 *                 }],
 *                 destinations: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *                 sourcePorts: [{
 *                     fromPort: 0,
 *                     toPort: 65535,
 *                 }],
 *                 sources: [{
 *                     addressDefinition: "0.0.0.0/0",
 *                 }],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Network Firewall TLS inspection configuration.
 *
 * Using `pulumi import`, import Network Firewall TLS Inspection Configuration using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_networkfirewall_tls_inspection_configuration.example arn:aws:network-firewall::<region>:<account_id>:tls-configuration/example
 */
export declare class TlsInspectionConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing TlsInspectionConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TlsInspectionConfigurationState, opts?: pulumi.CustomResourceOptions): TlsInspectionConfiguration;
    /**
     * Returns true if the given object is an instance of TlsInspectionConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TlsInspectionConfiguration;
    /**
     * ARN of the TLS Inspection Configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Certificate Manager certificate block. See Certificate Authority below for details.
     */
    readonly certificateAuthorities: pulumi.Output<outputs.networkfirewall.TlsInspectionConfigurationCertificateAuthority[]>;
    /**
     * List of certificate blocks describing certificates associated with the TLS inspection configuration. See Certificates below for details.
     */
    readonly certificates: pulumi.Output<outputs.networkfirewall.TlsInspectionConfigurationCertificate[]>;
    /**
     * Description of the TLS inspection configuration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Encryption configuration block. Detailed below.
     */
    readonly encryptionConfigurations: pulumi.Output<outputs.networkfirewall.TlsInspectionConfigurationEncryptionConfiguration[]>;
    /**
     * Descriptive name of the TLS inspection configuration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Number of firewall policies that use this TLS inspection configuration.
     */
    readonly numberOfAssociations: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.networkfirewall.TlsInspectionConfigurationTimeouts | undefined>;
    /**
     * TLS inspection configuration block. Detailed below.
     *
     * The following arguments are optional:
     */
    readonly tlsInspectionConfiguration: pulumi.Output<outputs.networkfirewall.TlsInspectionConfigurationTlsInspectionConfiguration | undefined>;
    /**
     * A unique identifier for the TLS inspection configuration.
     */
    readonly tlsInspectionConfigurationId: pulumi.Output<string>;
    /**
     * String token used when updating the rule group.
     */
    readonly updateToken: pulumi.Output<string>;
    /**
     * Create a TlsInspectionConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TlsInspectionConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TlsInspectionConfiguration resources.
 */
export interface TlsInspectionConfigurationState {
    /**
     * ARN of the TLS Inspection Configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Certificate Manager certificate block. See Certificate Authority below for details.
     */
    certificateAuthorities?: pulumi.Input<pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationCertificateAuthority>[]>;
    /**
     * List of certificate blocks describing certificates associated with the TLS inspection configuration. See Certificates below for details.
     */
    certificates?: pulumi.Input<pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationCertificate>[]>;
    /**
     * Description of the TLS inspection configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Encryption configuration block. Detailed below.
     */
    encryptionConfigurations?: pulumi.Input<pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationEncryptionConfiguration>[]>;
    /**
     * Descriptive name of the TLS inspection configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Number of firewall policies that use this TLS inspection configuration.
     */
    numberOfAssociations?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationTimeouts>;
    /**
     * TLS inspection configuration block. Detailed below.
     *
     * The following arguments are optional:
     */
    tlsInspectionConfiguration?: pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationTlsInspectionConfiguration>;
    /**
     * A unique identifier for the TLS inspection configuration.
     */
    tlsInspectionConfigurationId?: pulumi.Input<string>;
    /**
     * String token used when updating the rule group.
     */
    updateToken?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TlsInspectionConfiguration resource.
 */
export interface TlsInspectionConfigurationArgs {
    /**
     * Description of the TLS inspection configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Encryption configuration block. Detailed below.
     */
    encryptionConfigurations?: pulumi.Input<pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationEncryptionConfiguration>[]>;
    /**
     * Descriptive name of the TLS inspection configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationTimeouts>;
    /**
     * TLS inspection configuration block. Detailed below.
     *
     * The following arguments are optional:
     */
    tlsInspectionConfiguration?: pulumi.Input<inputs.networkfirewall.TlsInspectionConfigurationTlsInspectionConfiguration>;
}
