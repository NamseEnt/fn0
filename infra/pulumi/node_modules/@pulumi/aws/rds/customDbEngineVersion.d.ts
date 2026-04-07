import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an custom engine version (CEV) resource for Amazon RDS Custom. For additional information, see [Working with CEVs for RDS Custom for Oracle](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/custom-cev.html) and [Working with CEVs for RDS Custom for SQL Server](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/custom-cev-sqlserver.html) in the the [RDS User Guide](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Welcome.html).
 *
 * ## Example Usage
 *
 * ### RDS Custom for Oracle Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {description: "KMS symmetric key for RDS Custom for Oracle"});
 * const exampleCustomDbEngineVersion = new aws.rds.CustomDbEngineVersion("example", {
 *     databaseInstallationFilesS3BucketName: "DOC-EXAMPLE-BUCKET",
 *     databaseInstallationFilesS3Prefix: "1915_GI/",
 *     engine: "custom-oracle-ee-cdb",
 *     engineVersion: "19.cdb_cev1",
 *     kmsKeyId: example.arn,
 *     manifest: `  {
 * \\t\\"databaseInstallationFileNames\\":[\\"V982063-01.zip\\"]
 *   }
 * `,
 *     tags: {
 *         Name: "example",
 *         Key: "value",
 *     },
 * });
 * ```
 *
 * ### RDS Custom for Oracle External Manifest Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.kms.Key("example", {description: "KMS symmetric key for RDS Custom for Oracle"});
 * const exampleCustomDbEngineVersion = new aws.rds.CustomDbEngineVersion("example", {
 *     databaseInstallationFilesS3BucketName: "DOC-EXAMPLE-BUCKET",
 *     databaseInstallationFilesS3Prefix: "1915_GI/",
 *     engine: "custom-oracle-ee-cdb",
 *     engineVersion: "19.cdb_cev1",
 *     kmsKeyId: example.arn,
 *     filename: "manifest_1915_GI.json",
 *     manifestHash: std.filebase64sha256({
 *         input: json,
 *     }).then(invoke => invoke.result),
 *     tags: {
 *         Name: "example",
 *         Key: "value",
 *     },
 * });
 * ```
 *
 * ### RDS Custom for SQL Server Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // CEV creation requires an AMI owned by the operator
 * const test = new aws.rds.CustomDbEngineVersion("test", {
 *     engine: "custom-sqlserver-se",
 *     engineVersion: "15.00.4249.2.cev-1",
 *     sourceImageId: "ami-0aa12345678a12ab1",
 * });
 * ```
 *
 * ### RDS Custom for SQL Server Usage with AMI from another region
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.AmiCopy("example", {
 *     name: "sqlserver-se-2019-15.00.4249.2",
 *     description: "A copy of ami-xxxxxxxx",
 *     sourceAmiId: "ami-xxxxxxxx",
 *     sourceAmiRegion: "us-east-1",
 * });
 * // CEV creation requires an AMI owned by the operator
 * const test = new aws.rds.CustomDbEngineVersion("test", {
 *     engine: "custom-sqlserver-se",
 *     engineVersion: "15.00.4249.2.cev-1",
 *     sourceImageId: example.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import custom engine versions for Amazon RDS custom using the `engine` and `engine_version` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:rds/customDbEngineVersion:CustomDbEngineVersion example custom-oracle-ee-cdb:19.cdb_cev1
 * ```
 */
export declare class CustomDbEngineVersion extends pulumi.CustomResource {
    /**
     * Get an existing CustomDbEngineVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomDbEngineVersionState, opts?: pulumi.CustomResourceOptions): CustomDbEngineVersion;
    /**
     * Returns true if the given object is an instance of CustomDbEngineVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomDbEngineVersion;
    /**
     * The Amazon Resource Name (ARN) for the custom engine version.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date and time that the CEV was created.
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * The name of the Amazon S3 bucket that contains the database installation files.
     */
    readonly databaseInstallationFilesS3BucketName: pulumi.Output<string | undefined>;
    /**
     * The prefix for the Amazon S3 bucket that contains the database installation files.
     */
    readonly databaseInstallationFilesS3Prefix: pulumi.Output<string | undefined>;
    /**
     * The name of the DB parameter group family for the CEV.
     */
    readonly dbParameterGroupFamily: pulumi.Output<string>;
    /**
     * The description of the CEV.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the database engine. Valid values are `custom-oracle*`, `custom-sqlserver*`.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * The version of the database engine.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * The name of the manifest file within the local filesystem. Conflicts with `manifest`.
     */
    readonly filename: pulumi.Output<string | undefined>;
    /**
     * The ID of the AMI that was created with the CEV.
     */
    readonly imageId: pulumi.Output<string>;
    /**
     * The ARN of the AWS KMS key that is used to encrypt the database installation files. Required for RDS Custom for Oracle.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The major version of the database engine.
     */
    readonly majorEngineVersion: pulumi.Output<string>;
    /**
     * The manifest file, in JSON format, that contains the list of database installation files. Conflicts with `filename`.
     */
    readonly manifest: pulumi.Output<string | undefined>;
    /**
     * The returned manifest file, in JSON format, service generated and often different from input `manifest`.
     */
    readonly manifestComputed: pulumi.Output<string>;
    /**
     * Used to trigger updates. Must be set to a base64-encoded SHA256 hash of the manifest source specified with `filename`. The usual way to set this is filebase64sha256("manifest.json") where "manifest.json" is the local filename of the manifest source.
     */
    readonly manifestHash: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the AMI to create the CEV from. Required for RDS Custom for SQL Server. For RDS Custom for Oracle, you can specify an AMI ID that was used in a different Oracle CEV.
     */
    readonly sourceImageId: pulumi.Output<string | undefined>;
    /**
     * The status of the CEV. Valid values are `available`, `inactive`, `inactive-except-restore`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a CustomDbEngineVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomDbEngineVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomDbEngineVersion resources.
 */
export interface CustomDbEngineVersionState {
    /**
     * The Amazon Resource Name (ARN) for the custom engine version.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date and time that the CEV was created.
     */
    createTime?: pulumi.Input<string>;
    /**
     * The name of the Amazon S3 bucket that contains the database installation files.
     */
    databaseInstallationFilesS3BucketName?: pulumi.Input<string>;
    /**
     * The prefix for the Amazon S3 bucket that contains the database installation files.
     */
    databaseInstallationFilesS3Prefix?: pulumi.Input<string>;
    /**
     * The name of the DB parameter group family for the CEV.
     */
    dbParameterGroupFamily?: pulumi.Input<string>;
    /**
     * The description of the CEV.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the database engine. Valid values are `custom-oracle*`, `custom-sqlserver*`.
     */
    engine?: pulumi.Input<string>;
    /**
     * The version of the database engine.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * The name of the manifest file within the local filesystem. Conflicts with `manifest`.
     */
    filename?: pulumi.Input<string>;
    /**
     * The ID of the AMI that was created with the CEV.
     */
    imageId?: pulumi.Input<string>;
    /**
     * The ARN of the AWS KMS key that is used to encrypt the database installation files. Required for RDS Custom for Oracle.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The major version of the database engine.
     */
    majorEngineVersion?: pulumi.Input<string>;
    /**
     * The manifest file, in JSON format, that contains the list of database installation files. Conflicts with `filename`.
     */
    manifest?: pulumi.Input<string>;
    /**
     * The returned manifest file, in JSON format, service generated and often different from input `manifest`.
     */
    manifestComputed?: pulumi.Input<string>;
    /**
     * Used to trigger updates. Must be set to a base64-encoded SHA256 hash of the manifest source specified with `filename`. The usual way to set this is filebase64sha256("manifest.json") where "manifest.json" is the local filename of the manifest source.
     */
    manifestHash?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the AMI to create the CEV from. Required for RDS Custom for SQL Server. For RDS Custom for Oracle, you can specify an AMI ID that was used in a different Oracle CEV.
     */
    sourceImageId?: pulumi.Input<string>;
    /**
     * The status of the CEV. Valid values are `available`, `inactive`, `inactive-except-restore`.
     */
    status?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a CustomDbEngineVersion resource.
 */
export interface CustomDbEngineVersionArgs {
    /**
     * The name of the Amazon S3 bucket that contains the database installation files.
     */
    databaseInstallationFilesS3BucketName?: pulumi.Input<string>;
    /**
     * The prefix for the Amazon S3 bucket that contains the database installation files.
     */
    databaseInstallationFilesS3Prefix?: pulumi.Input<string>;
    /**
     * The description of the CEV.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the database engine. Valid values are `custom-oracle*`, `custom-sqlserver*`.
     */
    engine: pulumi.Input<string>;
    /**
     * The version of the database engine.
     */
    engineVersion: pulumi.Input<string>;
    /**
     * The name of the manifest file within the local filesystem. Conflicts with `manifest`.
     */
    filename?: pulumi.Input<string>;
    /**
     * The ARN of the AWS KMS key that is used to encrypt the database installation files. Required for RDS Custom for Oracle.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The manifest file, in JSON format, that contains the list of database installation files. Conflicts with `filename`.
     */
    manifest?: pulumi.Input<string>;
    /**
     * Used to trigger updates. Must be set to a base64-encoded SHA256 hash of the manifest source specified with `filename`. The usual way to set this is filebase64sha256("manifest.json") where "manifest.json" is the local filename of the manifest source.
     */
    manifestHash?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the AMI to create the CEV from. Required for RDS Custom for SQL Server. For RDS Custom for Oracle, you can specify an AMI ID that was used in a different Oracle CEV.
     */
    sourceImageId?: pulumi.Input<string>;
    /**
     * The status of the CEV. Valid values are `available`, `inactive`, `inactive-except-restore`.
     */
    status?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
