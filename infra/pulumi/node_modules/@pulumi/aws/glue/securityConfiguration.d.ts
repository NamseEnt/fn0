import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Glue Security Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.SecurityConfiguration("example", {
 *     name: "example",
 *     encryptionConfiguration: {
 *         cloudwatchEncryption: {
 *             cloudwatchEncryptionMode: "DISABLED",
 *         },
 *         jobBookmarksEncryption: {
 *             jobBookmarksEncryptionMode: "DISABLED",
 *         },
 *         s3Encryption: {
 *             kmsKeyArn: exampleAwsKmsKey.arn,
 *             s3EncryptionMode: "SSE-KMS",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Security Configurations using `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/securityConfiguration:SecurityConfiguration example example
 * ```
 */
export declare class SecurityConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing SecurityConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecurityConfigurationState, opts?: pulumi.CustomResourceOptions): SecurityConfiguration;
    /**
     * Returns true if the given object is an instance of SecurityConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecurityConfiguration;
    /**
     * Configuration block containing encryption configuration. Detailed below.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.glue.SecurityConfigurationEncryptionConfiguration>;
    /**
     * Name of the security configuration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a SecurityConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecurityConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecurityConfiguration resources.
 */
export interface SecurityConfigurationState {
    /**
     * Configuration block containing encryption configuration. Detailed below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.glue.SecurityConfigurationEncryptionConfiguration>;
    /**
     * Name of the security configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SecurityConfiguration resource.
 */
export interface SecurityConfigurationArgs {
    /**
     * Configuration block containing encryption configuration. Detailed below.
     */
    encryptionConfiguration: pulumi.Input<inputs.glue.SecurityConfigurationEncryptionConfiguration>;
    /**
     * Name of the security configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
