import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a AWS Transfer AS2 Connector resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.Connector("example", {
 *     accessRole: test.arn,
 *     as2Config: {
 *         compression: "DISABLED",
 *         encryptionAlgorithm: "AWS128_CBC",
 *         messageSubject: "For Connector",
 *         localProfileId: local.profileId,
 *         mdnResponse: "NONE",
 *         mdnSigningAlgorithm: "NONE",
 *         partnerProfileId: partner.profileId,
 *         signingAlgorithm: "NONE",
 *     },
 *     url: "http://www.test.com",
 * });
 * ```
 *
 * ### SFTP Connector
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.Connector("example", {
 *     accessRole: test.arn,
 *     sftpConfig: {
 *         trustedHostKeys: ["ssh-rsa AAAAB3NYourKeysHere"],
 *         userSecretId: exampleAwsSecretsmanagerSecret.id,
 *     },
 *     url: "sftp://test.com",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer AS2 Connector using the `connector_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/connector:Connector example c-4221a88afd5f4362a
 * ```
 */
export declare class Connector extends pulumi.CustomResource {
    /**
     * Get an existing Connector resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectorState, opts?: pulumi.CustomResourceOptions): Connector;
    /**
     * Returns true if the given object is an instance of Connector.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Connector;
    /**
     * The IAM Role which provides read and write access to the parent directory of the file location mentioned in the StartFileTransfer request.
     */
    readonly accessRole: pulumi.Output<string>;
    /**
     * The ARN of the connector.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Either SFTP or AS2 is configured.The parameters to configure for the connector object. Fields documented below.
     */
    readonly as2Config: pulumi.Output<outputs.transfer.ConnectorAs2Config | undefined>;
    /**
     * The unique identifier for the AS2 profile or SFTP Profile.
     */
    readonly connectorId: pulumi.Output<string>;
    /**
     * The IAM Role which is required for allowing the connector to turn on CloudWatch logging for Amazon S3 events.
     */
    readonly loggingRole: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the security policy for the connector.
     */
    readonly securityPolicyName: pulumi.Output<string>;
    /**
     * Either SFTP or AS2 is configured.The parameters to configure for the connector object. Fields documented below.
     */
    readonly sftpConfig: pulumi.Output<outputs.transfer.ConnectorSftpConfig | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The URL of the partners AS2 endpoint or SFTP endpoint.
     */
    readonly url: pulumi.Output<string>;
    /**
     * Create a Connector resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Connector resources.
 */
export interface ConnectorState {
    /**
     * The IAM Role which provides read and write access to the parent directory of the file location mentioned in the StartFileTransfer request.
     */
    accessRole?: pulumi.Input<string>;
    /**
     * The ARN of the connector.
     */
    arn?: pulumi.Input<string>;
    /**
     * Either SFTP or AS2 is configured.The parameters to configure for the connector object. Fields documented below.
     */
    as2Config?: pulumi.Input<inputs.transfer.ConnectorAs2Config>;
    /**
     * The unique identifier for the AS2 profile or SFTP Profile.
     */
    connectorId?: pulumi.Input<string>;
    /**
     * The IAM Role which is required for allowing the connector to turn on CloudWatch logging for Amazon S3 events.
     */
    loggingRole?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the security policy for the connector.
     */
    securityPolicyName?: pulumi.Input<string>;
    /**
     * Either SFTP or AS2 is configured.The parameters to configure for the connector object. Fields documented below.
     */
    sftpConfig?: pulumi.Input<inputs.transfer.ConnectorSftpConfig>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The URL of the partners AS2 endpoint or SFTP endpoint.
     */
    url?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Connector resource.
 */
export interface ConnectorArgs {
    /**
     * The IAM Role which provides read and write access to the parent directory of the file location mentioned in the StartFileTransfer request.
     */
    accessRole: pulumi.Input<string>;
    /**
     * Either SFTP or AS2 is configured.The parameters to configure for the connector object. Fields documented below.
     */
    as2Config?: pulumi.Input<inputs.transfer.ConnectorAs2Config>;
    /**
     * The IAM Role which is required for allowing the connector to turn on CloudWatch logging for Amazon S3 events.
     */
    loggingRole?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the security policy for the connector.
     */
    securityPolicyName?: pulumi.Input<string>;
    /**
     * Either SFTP or AS2 is configured.The parameters to configure for the connector object. Fields documented below.
     */
    sftpConfig?: pulumi.Input<inputs.transfer.ConnectorSftpConfig>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The URL of the partners AS2 endpoint or SFTP endpoint.
     */
    url: pulumi.Input<string>;
}
