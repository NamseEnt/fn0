import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * This data source can be used to fetch information about AWS Glue Data Catalog Encryption Settings.
 *
 * ## Example Usage
 */
export declare function getDataCatalogEncryptionSettings(args: GetDataCatalogEncryptionSettingsArgs, opts?: pulumi.InvokeOptions): Promise<GetDataCatalogEncryptionSettingsResult>;
/**
 * A collection of arguments for invoking getDataCatalogEncryptionSettings.
 */
export interface GetDataCatalogEncryptionSettingsArgs {
    /**
     * ID of the Data Catalog. This is typically the AWS account ID.
     */
    catalogId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDataCatalogEncryptionSettings.
 */
export interface GetDataCatalogEncryptionSettingsResult {
    readonly catalogId: string;
    /**
     * The security configuration to set. see Data Catalog Encryption Settings.
     */
    readonly dataCatalogEncryptionSettings: outputs.glue.GetDataCatalogEncryptionSettingsDataCatalogEncryptionSetting[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * This data source can be used to fetch information about AWS Glue Data Catalog Encryption Settings.
 *
 * ## Example Usage
 */
export declare function getDataCatalogEncryptionSettingsOutput(args: GetDataCatalogEncryptionSettingsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDataCatalogEncryptionSettingsResult>;
/**
 * A collection of arguments for invoking getDataCatalogEncryptionSettings.
 */
export interface GetDataCatalogEncryptionSettingsOutputArgs {
    /**
     * ID of the Data Catalog. This is typically the AWS account ID.
     */
    catalogId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
