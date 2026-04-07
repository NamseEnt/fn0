import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Athena database.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example"});
 * const exampleDatabase = new aws.athena.Database("example", {
 *     name: "database_name",
 *     bucket: example.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Athena Databases using their name. For example:
 *
 * ```sh
 * $ pulumi import aws:athena/database:Database example example
 * ```
 * Certain resource arguments, like `encryption_configuration` and `bucket`, do not have an API method for reading the information after creation. If the argument is set in the Pulumi program on an imported resource, Pulumi will always show a difference. To workaround this behavior, either omit the argument from the Pulumi program or use `ignore_changes` to hide the difference. For example:
 */
export declare class Database extends pulumi.CustomResource {
    /**
     * Get an existing Database resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DatabaseState, opts?: pulumi.CustomResourceOptions): Database;
    /**
     * Returns true if the given object is an instance of Database.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Database;
    /**
     * That an Amazon S3 canned ACL should be set to control ownership of stored query results. See ACL Configuration below.
     */
    readonly aclConfiguration: pulumi.Output<outputs.athena.DatabaseAclConfiguration | undefined>;
    /**
     * Name of S3 bucket to save the results of the query execution.
     */
    readonly bucket: pulumi.Output<string | undefined>;
    /**
     * Description of the database.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * Encryption key block AWS Athena uses to decrypt the data in S3, such as an AWS Key Management Service (AWS KMS) key. See Encryption Configuration below.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.athena.DatabaseEncryptionConfiguration | undefined>;
    /**
     * AWS account ID that you expect to be the owner of the Amazon S3 bucket.
     */
    readonly expectedBucketOwner: pulumi.Output<string | undefined>;
    /**
     * Boolean that indicates all tables should be deleted from the database so that the database can be destroyed without error. The tables are *not* recoverable.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Name of the database to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Key-value map of custom metadata properties for the database definition.
     */
    readonly properties: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the workgroup.
     */
    readonly workgroup: pulumi.Output<string | undefined>;
    /**
     * Create a Database resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DatabaseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Database resources.
 */
export interface DatabaseState {
    /**
     * That an Amazon S3 canned ACL should be set to control ownership of stored query results. See ACL Configuration below.
     */
    aclConfiguration?: pulumi.Input<inputs.athena.DatabaseAclConfiguration>;
    /**
     * Name of S3 bucket to save the results of the query execution.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Description of the database.
     */
    comment?: pulumi.Input<string>;
    /**
     * Encryption key block AWS Athena uses to decrypt the data in S3, such as an AWS Key Management Service (AWS KMS) key. See Encryption Configuration below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.athena.DatabaseEncryptionConfiguration>;
    /**
     * AWS account ID that you expect to be the owner of the Amazon S3 bucket.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Boolean that indicates all tables should be deleted from the database so that the database can be destroyed without error. The tables are *not* recoverable.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Name of the database to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Key-value map of custom metadata properties for the database definition.
     */
    properties?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the workgroup.
     */
    workgroup?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Database resource.
 */
export interface DatabaseArgs {
    /**
     * That an Amazon S3 canned ACL should be set to control ownership of stored query results. See ACL Configuration below.
     */
    aclConfiguration?: pulumi.Input<inputs.athena.DatabaseAclConfiguration>;
    /**
     * Name of S3 bucket to save the results of the query execution.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Description of the database.
     */
    comment?: pulumi.Input<string>;
    /**
     * Encryption key block AWS Athena uses to decrypt the data in S3, such as an AWS Key Management Service (AWS KMS) key. See Encryption Configuration below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.athena.DatabaseEncryptionConfiguration>;
    /**
     * AWS account ID that you expect to be the owner of the Amazon S3 bucket.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Boolean that indicates all tables should be deleted from the database so that the database can be destroyed without error. The tables are *not* recoverable.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Name of the database to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Key-value map of custom metadata properties for the database definition.
     */
    properties?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the workgroup.
     */
    workgroup?: pulumi.Input<string>;
}
