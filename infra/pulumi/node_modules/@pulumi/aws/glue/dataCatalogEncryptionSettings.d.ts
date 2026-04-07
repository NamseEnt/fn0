import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue Data Catalog Encryption Settings resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.DataCatalogEncryptionSettings("example", {dataCatalogEncryptionSettings: {
 *     connectionPasswordEncryption: {
 *         awsKmsKeyId: test.arn,
 *         returnConnectionPasswordEncrypted: true,
 *     },
 *     encryptionAtRest: {
 *         catalogEncryptionMode: "SSE-KMS",
 *         catalogEncryptionServiceRole: role.test.arn,
 *         sseAwsKmsKeyId: test.arn,
 *     },
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Data Catalog Encryption Settings using `CATALOG-ID` (AWS account ID if not custom). For example:
 *
 * ```sh
 * $ pulumi import aws:glue/dataCatalogEncryptionSettings:DataCatalogEncryptionSettings example 123456789012
 * ```
 */
export declare class DataCatalogEncryptionSettings extends pulumi.CustomResource {
    /**
     * Get an existing DataCatalogEncryptionSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataCatalogEncryptionSettingsState, opts?: pulumi.CustomResourceOptions): DataCatalogEncryptionSettings;
    /**
     * Returns true if the given object is an instance of DataCatalogEncryptionSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataCatalogEncryptionSettings;
    /**
     * The ID of the Data Catalog to set the security configuration for. If none is provided, the AWS account ID is used by default.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * The security configuration to set. see Data Catalog Encryption Settings.
     */
    readonly dataCatalogEncryptionSettings: pulumi.Output<outputs.glue.DataCatalogEncryptionSettingsDataCatalogEncryptionSettings>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DataCatalogEncryptionSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataCatalogEncryptionSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataCatalogEncryptionSettings resources.
 */
export interface DataCatalogEncryptionSettingsState {
    /**
     * The ID of the Data Catalog to set the security configuration for. If none is provided, the AWS account ID is used by default.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * The security configuration to set. see Data Catalog Encryption Settings.
     */
    dataCatalogEncryptionSettings?: pulumi.Input<inputs.glue.DataCatalogEncryptionSettingsDataCatalogEncryptionSettings>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DataCatalogEncryptionSettings resource.
 */
export interface DataCatalogEncryptionSettingsArgs {
    /**
     * The ID of the Data Catalog to set the security configuration for. If none is provided, the AWS account ID is used by default.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * The security configuration to set. see Data Catalog Encryption Settings.
     */
    dataCatalogEncryptionSettings: pulumi.Input<inputs.glue.DataCatalogEncryptionSettingsDataCatalogEncryptionSettings>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
