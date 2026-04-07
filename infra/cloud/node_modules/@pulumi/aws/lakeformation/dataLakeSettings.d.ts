import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages Lake Formation principals designated as data lake administrators and lists of principal permission entries for default create database and default create table permissions.
 *
 * > **NOTE:** Lake Formation introduces fine-grained access control for data in your data lake. Part of the changes include the `IAMAllowedPrincipals` principal in order to make Lake Formation backwards compatible with existing IAM and Glue permissions. For more information, see [Changing the Default Security Settings for Your Data Lake](https://docs.aws.amazon.com/lake-formation/latest/dg/change-settings.html) and [Upgrading AWS Glue Data Permissions to the AWS Lake Formation Model](https://docs.aws.amazon.com/lake-formation/latest/dg/upgrade-glue-lake-formation.html).
 *
 * ## Example Usage
 *
 * ### Data Lake Admins
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.DataLakeSettings("example", {admins: [
 *     test.arn,
 *     testAwsIamRole.arn,
 * ]});
 * ```
 *
 * ### Create Default Permissions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.DataLakeSettings("example", {
 *     admins: [
 *         test.arn,
 *         testAwsIamRole.arn,
 *     ],
 *     createDatabaseDefaultPermissions: [{
 *         permissions: [
 *             "SELECT",
 *             "ALTER",
 *             "DROP",
 *         ],
 *         principal: test.arn,
 *     }],
 *     createTableDefaultPermissions: [{
 *         permissions: ["ALL"],
 *         principal: testAwsIamRole.arn,
 *     }],
 * });
 * ```
 *
 * ### Enable EMR access to LakeFormation resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.DataLakeSettings("example", {
 *     admins: [
 *         test.arn,
 *         testAwsIamRole.arn,
 *     ],
 *     createDatabaseDefaultPermissions: [{
 *         permissions: [
 *             "SELECT",
 *             "ALTER",
 *             "DROP",
 *         ],
 *         principal: test.arn,
 *     }],
 *     createTableDefaultPermissions: [{
 *         permissions: ["ALL"],
 *         principal: testAwsIamRole.arn,
 *     }],
 *     allowExternalDataFiltering: true,
 *     externalDataFilteringAllowLists: [
 *         current.accountId,
 *         thirdParty.accountId,
 *     ],
 *     authorizedSessionTagValueLists: ["Amazon EMR"],
 *     allowFullTableExternalDataAccess: true,
 * });
 * ```
 *
 * ### Change Cross Account Version
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.DataLakeSettings("example", {parameters: {
 *     CROSS_ACCOUNT_VERSION: "3",
 * }});
 * ```
 */
export declare class DataLakeSettings extends pulumi.CustomResource {
    /**
     * Get an existing DataLakeSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataLakeSettingsState, opts?: pulumi.CustomResourceOptions): DataLakeSettings;
    /**
     * Returns true if the given object is an instance of DataLakeSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataLakeSettings;
    /**
     * Set of ARNs of AWS Lake Formation principals (IAM users or roles).
     */
    readonly admins: pulumi.Output<string[]>;
    /**
     * Whether to allow Amazon EMR clusters to access data managed by Lake Formation.
     */
    readonly allowExternalDataFiltering: pulumi.Output<boolean | undefined>;
    /**
     * Whether to allow a third-party query engine to get data access credentials without session tags when a caller has full data access permissions.
     */
    readonly allowFullTableExternalDataAccess: pulumi.Output<boolean | undefined>;
    /**
     * Lake Formation relies on a privileged process secured by Amazon EMR or the third party integrator to tag the user's role while assuming it.
     */
    readonly authorizedSessionTagValueLists: pulumi.Output<string[]>;
    /**
     * Identifier for the Data Catalog. By default, the account ID.
     */
    readonly catalogId: pulumi.Output<string | undefined>;
    /**
     * Up to three configuration blocks of principal permissions for default create database permissions. Detailed below.
     */
    readonly createDatabaseDefaultPermissions: pulumi.Output<outputs.lakeformation.DataLakeSettingsCreateDatabaseDefaultPermission[]>;
    /**
     * Up to three configuration blocks of principal permissions for default create table permissions. Detailed below.
     */
    readonly createTableDefaultPermissions: pulumi.Output<outputs.lakeformation.DataLakeSettingsCreateTableDefaultPermission[]>;
    /**
     * A list of the account IDs of Amazon Web Services accounts with Amazon EMR clusters that are to perform data filtering.
     */
    readonly externalDataFilteringAllowLists: pulumi.Output<string[]>;
    /**
     * Key-value map of additional configuration. Valid values for the `CROSS_ACCOUNT_VERSION` key are `"1"`, `"2"`, `"3"`, or `"4"`. `SET_CONTEXT` is also returned with a value of `TRUE`. In a fresh account, prior to configuring, `CROSS_ACCOUNT_VERSION` is `"1"`. Destroying this resource sets the `CROSS_ACCOUNT_VERSION` to `"1"`.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Set of ARNs of AWS Lake Formation principals (IAM users or roles) with only view access to the resources.
     */
    readonly readOnlyAdmins: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of the resource-owning account IDs that the caller's account can use to share their user access details (user ARNs).
     *
     * > **NOTE:** Although optional, not including `admins`, `createDatabaseDefaultPermissions`, `createTableDefaultPermissions`, `parameters`, and/or `trustedResourceOwners` results in the setting being cleared.
     */
    readonly trustedResourceOwners: pulumi.Output<string[]>;
    /**
     * Create a DataLakeSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DataLakeSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataLakeSettings resources.
 */
export interface DataLakeSettingsState {
    /**
     * Set of ARNs of AWS Lake Formation principals (IAM users or roles).
     */
    admins?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to allow Amazon EMR clusters to access data managed by Lake Formation.
     */
    allowExternalDataFiltering?: pulumi.Input<boolean>;
    /**
     * Whether to allow a third-party query engine to get data access credentials without session tags when a caller has full data access permissions.
     */
    allowFullTableExternalDataAccess?: pulumi.Input<boolean>;
    /**
     * Lake Formation relies on a privileged process secured by Amazon EMR or the third party integrator to tag the user's role while assuming it.
     */
    authorizedSessionTagValueLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Identifier for the Data Catalog. By default, the account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Up to three configuration blocks of principal permissions for default create database permissions. Detailed below.
     */
    createDatabaseDefaultPermissions?: pulumi.Input<pulumi.Input<inputs.lakeformation.DataLakeSettingsCreateDatabaseDefaultPermission>[]>;
    /**
     * Up to three configuration blocks of principal permissions for default create table permissions. Detailed below.
     */
    createTableDefaultPermissions?: pulumi.Input<pulumi.Input<inputs.lakeformation.DataLakeSettingsCreateTableDefaultPermission>[]>;
    /**
     * A list of the account IDs of Amazon Web Services accounts with Amazon EMR clusters that are to perform data filtering.
     */
    externalDataFilteringAllowLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of additional configuration. Valid values for the `CROSS_ACCOUNT_VERSION` key are `"1"`, `"2"`, `"3"`, or `"4"`. `SET_CONTEXT` is also returned with a value of `TRUE`. In a fresh account, prior to configuring, `CROSS_ACCOUNT_VERSION` is `"1"`. Destroying this resource sets the `CROSS_ACCOUNT_VERSION` to `"1"`.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Set of ARNs of AWS Lake Formation principals (IAM users or roles) with only view access to the resources.
     */
    readOnlyAdmins?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of the resource-owning account IDs that the caller's account can use to share their user access details (user ARNs).
     *
     * > **NOTE:** Although optional, not including `admins`, `createDatabaseDefaultPermissions`, `createTableDefaultPermissions`, `parameters`, and/or `trustedResourceOwners` results in the setting being cleared.
     */
    trustedResourceOwners?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a DataLakeSettings resource.
 */
export interface DataLakeSettingsArgs {
    /**
     * Set of ARNs of AWS Lake Formation principals (IAM users or roles).
     */
    admins?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to allow Amazon EMR clusters to access data managed by Lake Formation.
     */
    allowExternalDataFiltering?: pulumi.Input<boolean>;
    /**
     * Whether to allow a third-party query engine to get data access credentials without session tags when a caller has full data access permissions.
     */
    allowFullTableExternalDataAccess?: pulumi.Input<boolean>;
    /**
     * Lake Formation relies on a privileged process secured by Amazon EMR or the third party integrator to tag the user's role while assuming it.
     */
    authorizedSessionTagValueLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Identifier for the Data Catalog. By default, the account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Up to three configuration blocks of principal permissions for default create database permissions. Detailed below.
     */
    createDatabaseDefaultPermissions?: pulumi.Input<pulumi.Input<inputs.lakeformation.DataLakeSettingsCreateDatabaseDefaultPermission>[]>;
    /**
     * Up to three configuration blocks of principal permissions for default create table permissions. Detailed below.
     */
    createTableDefaultPermissions?: pulumi.Input<pulumi.Input<inputs.lakeformation.DataLakeSettingsCreateTableDefaultPermission>[]>;
    /**
     * A list of the account IDs of Amazon Web Services accounts with Amazon EMR clusters that are to perform data filtering.
     */
    externalDataFilteringAllowLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of additional configuration. Valid values for the `CROSS_ACCOUNT_VERSION` key are `"1"`, `"2"`, `"3"`, or `"4"`. `SET_CONTEXT` is also returned with a value of `TRUE`. In a fresh account, prior to configuring, `CROSS_ACCOUNT_VERSION` is `"1"`. Destroying this resource sets the `CROSS_ACCOUNT_VERSION` to `"1"`.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Set of ARNs of AWS Lake Formation principals (IAM users or roles) with only view access to the resources.
     */
    readOnlyAdmins?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of the resource-owning account IDs that the caller's account can use to share their user access details (user ARNs).
     *
     * > **NOTE:** Although optional, not including `admins`, `createDatabaseDefaultPermissions`, `createTableDefaultPermissions`, `parameters`, and/or `trustedResourceOwners` results in the setting being cleared.
     */
    trustedResourceOwners?: pulumi.Input<pulumi.Input<string>[]>;
}
