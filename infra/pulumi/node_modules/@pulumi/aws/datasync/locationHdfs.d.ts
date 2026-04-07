import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an HDFS Location within AWS DataSync.
 *
 * > **NOTE:** The DataSync Agents must be available before creating this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.LocationHdfs("example", {
 *     agentArns: [exampleAwsDatasyncAgent.arn],
 *     authenticationType: "SIMPLE",
 *     simpleUser: "example",
 *     nameNodes: [{
 *         hostname: exampleAwsInstance.privateDns,
 *         port: 80,
 *     }],
 * });
 * ```
 *
 * ### Kerberos Authentication
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.datasync.LocationHdfs("example", {
 *     agentArns: [exampleAwsDatasyncAgent.arn],
 *     authenticationType: "KERBEROS",
 *     nameNodes: [{
 *         hostname: exampleAwsInstance.privateDns,
 *         port: 80,
 *     }],
 *     kerberosPrincipal: "user@example.com",
 *     kerberosKeytabBase64: std.filebase64({
 *         input: "user.keytab",
 *     }).then(invoke => invoke.result),
 *     kerberosKrb5Conf: std.file({
 *         input: "krb5.conf",
 *     }).then(invoke => invoke.result),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DataSync HDFS location.
 *
 * Using `pulumi import`, import `aws_datasync_location_hdfs` using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_datasync_location_hdfs.example arn:aws:datasync:us-east-1:123456789012:location/loc-12345678901234567
 */
export declare class LocationHdfs extends pulumi.CustomResource {
    /**
     * Get an existing LocationHdfs resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LocationHdfsState, opts?: pulumi.CustomResourceOptions): LocationHdfs;
    /**
     * Returns true if the given object is an instance of LocationHdfs.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LocationHdfs;
    /**
     * A list of DataSync Agent ARNs with which this location will be associated.
     */
    readonly agentArns: pulumi.Output<string[]>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The type of authentication used to determine the identity of the user. Valid values are `SIMPLE` and `KERBEROS`.
     */
    readonly authenticationType: pulumi.Output<string | undefined>;
    /**
     * The size of data blocks to write into the HDFS cluster. The block size must be a multiple of 512 bytes. The default block size is 128 mebibytes (MiB).
     */
    readonly blockSize: pulumi.Output<number | undefined>;
    /**
     * The Kerberos key table (keytab) that contains mappings between the defined Kerberos principal and the encrypted keys. Use `kerberosKeytabBase64` instead whenever the value is not a valid UTF-8 string. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKeytabBase64`) is required.
     */
    readonly kerberosKeytab: pulumi.Output<string | undefined>;
    /**
     * Use instead of `kerberosKeytab` to pass base64-encoded binary data directly. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKeytab`) is required.
     */
    readonly kerberosKeytabBase64: pulumi.Output<string | undefined>;
    /**
     * The krb5.conf file that contains the Kerberos configuration information. Use `kerberosKrb5ConfBase64` instead whenever the value is not a valid UTF-8 string. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKrb5ConfBase64`) is required.
     */
    readonly kerberosKrb5Conf: pulumi.Output<string | undefined>;
    /**
     * Use instead of `kerberosKrb5Conf` to pass base64-encoded binary data directly. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKrb5Conf`) is required.
     */
    readonly kerberosKrb5ConfBase64: pulumi.Output<string | undefined>;
    /**
     * The Kerberos principal with access to the files and folders on the HDFS cluster. If `KERBEROS` is specified for `authenticationType`, this parameter is required.
     */
    readonly kerberosPrincipal: pulumi.Output<string | undefined>;
    /**
     * The URI of the HDFS cluster's Key Management Server (KMS).
     */
    readonly kmsKeyProviderUri: pulumi.Output<string | undefined>;
    /**
     * The NameNode that manages the HDFS namespace. The NameNode performs operations such as opening, closing, and renaming files and directories. The NameNode contains the information to map blocks of data to the DataNodes. You can use only one NameNode. See configuration below.
     */
    readonly nameNodes: pulumi.Output<outputs.datasync.LocationHdfsNameNode[]>;
    /**
     * The Quality of Protection (QOP) configuration specifies the Remote Procedure Call (RPC) and data transfer protection settings configured on the Hadoop Distributed File System (HDFS) cluster. If `qopConfiguration` isn't specified, `rpcProtection` and `dataTransferProtection` default to `PRIVACY`. If you set RpcProtection or DataTransferProtection, the other parameter assumes the same value.  See configuration below.
     */
    readonly qopConfiguration: pulumi.Output<outputs.datasync.LocationHdfsQopConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The number of DataNodes to replicate the data to when writing to the HDFS cluster. By default, data is replicated to three DataNodes.
     */
    readonly replicationFactor: pulumi.Output<number | undefined>;
    /**
     * The user name used to identify the client on the host operating system. If `SIMPLE` is specified for `authenticationType`, this parameter is required.
     */
    readonly simpleUser: pulumi.Output<string | undefined>;
    /**
     * A subdirectory in the HDFS cluster. This subdirectory is used to read data from or write data to the HDFS cluster. If the subdirectory isn't specified, it will default to /.
     */
    readonly subdirectory: pulumi.Output<string | undefined>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly uri: pulumi.Output<string>;
    /**
     * Create a LocationHdfs resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LocationHdfsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LocationHdfs resources.
 */
export interface LocationHdfsState {
    /**
     * A list of DataSync Agent ARNs with which this location will be associated.
     */
    agentArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    arn?: pulumi.Input<string>;
    /**
     * The type of authentication used to determine the identity of the user. Valid values are `SIMPLE` and `KERBEROS`.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * The size of data blocks to write into the HDFS cluster. The block size must be a multiple of 512 bytes. The default block size is 128 mebibytes (MiB).
     */
    blockSize?: pulumi.Input<number>;
    /**
     * The Kerberos key table (keytab) that contains mappings between the defined Kerberos principal and the encrypted keys. Use `kerberosKeytabBase64` instead whenever the value is not a valid UTF-8 string. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKeytabBase64`) is required.
     */
    kerberosKeytab?: pulumi.Input<string>;
    /**
     * Use instead of `kerberosKeytab` to pass base64-encoded binary data directly. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKeytab`) is required.
     */
    kerberosKeytabBase64?: pulumi.Input<string>;
    /**
     * The krb5.conf file that contains the Kerberos configuration information. Use `kerberosKrb5ConfBase64` instead whenever the value is not a valid UTF-8 string. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKrb5ConfBase64`) is required.
     */
    kerberosKrb5Conf?: pulumi.Input<string>;
    /**
     * Use instead of `kerberosKrb5Conf` to pass base64-encoded binary data directly. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKrb5Conf`) is required.
     */
    kerberosKrb5ConfBase64?: pulumi.Input<string>;
    /**
     * The Kerberos principal with access to the files and folders on the HDFS cluster. If `KERBEROS` is specified for `authenticationType`, this parameter is required.
     */
    kerberosPrincipal?: pulumi.Input<string>;
    /**
     * The URI of the HDFS cluster's Key Management Server (KMS).
     */
    kmsKeyProviderUri?: pulumi.Input<string>;
    /**
     * The NameNode that manages the HDFS namespace. The NameNode performs operations such as opening, closing, and renaming files and directories. The NameNode contains the information to map blocks of data to the DataNodes. You can use only one NameNode. See configuration below.
     */
    nameNodes?: pulumi.Input<pulumi.Input<inputs.datasync.LocationHdfsNameNode>[]>;
    /**
     * The Quality of Protection (QOP) configuration specifies the Remote Procedure Call (RPC) and data transfer protection settings configured on the Hadoop Distributed File System (HDFS) cluster. If `qopConfiguration` isn't specified, `rpcProtection` and `dataTransferProtection` default to `PRIVACY`. If you set RpcProtection or DataTransferProtection, the other parameter assumes the same value.  See configuration below.
     */
    qopConfiguration?: pulumi.Input<inputs.datasync.LocationHdfsQopConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The number of DataNodes to replicate the data to when writing to the HDFS cluster. By default, data is replicated to three DataNodes.
     */
    replicationFactor?: pulumi.Input<number>;
    /**
     * The user name used to identify the client on the host operating system. If `SIMPLE` is specified for `authenticationType`, this parameter is required.
     */
    simpleUser?: pulumi.Input<string>;
    /**
     * A subdirectory in the HDFS cluster. This subdirectory is used to read data from or write data to the HDFS cluster. If the subdirectory isn't specified, it will default to /.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    uri?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LocationHdfs resource.
 */
export interface LocationHdfsArgs {
    /**
     * A list of DataSync Agent ARNs with which this location will be associated.
     */
    agentArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of authentication used to determine the identity of the user. Valid values are `SIMPLE` and `KERBEROS`.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * The size of data blocks to write into the HDFS cluster. The block size must be a multiple of 512 bytes. The default block size is 128 mebibytes (MiB).
     */
    blockSize?: pulumi.Input<number>;
    /**
     * The Kerberos key table (keytab) that contains mappings between the defined Kerberos principal and the encrypted keys. Use `kerberosKeytabBase64` instead whenever the value is not a valid UTF-8 string. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKeytabBase64`) is required.
     */
    kerberosKeytab?: pulumi.Input<string>;
    /**
     * Use instead of `kerberosKeytab` to pass base64-encoded binary data directly. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKeytab`) is required.
     */
    kerberosKeytabBase64?: pulumi.Input<string>;
    /**
     * The krb5.conf file that contains the Kerberos configuration information. Use `kerberosKrb5ConfBase64` instead whenever the value is not a valid UTF-8 string. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKrb5ConfBase64`) is required.
     */
    kerberosKrb5Conf?: pulumi.Input<string>;
    /**
     * Use instead of `kerberosKrb5Conf` to pass base64-encoded binary data directly. If `KERBEROS` is specified for `authenticationType`, this parameter (or `kerberosKrb5Conf`) is required.
     */
    kerberosKrb5ConfBase64?: pulumi.Input<string>;
    /**
     * The Kerberos principal with access to the files and folders on the HDFS cluster. If `KERBEROS` is specified for `authenticationType`, this parameter is required.
     */
    kerberosPrincipal?: pulumi.Input<string>;
    /**
     * The URI of the HDFS cluster's Key Management Server (KMS).
     */
    kmsKeyProviderUri?: pulumi.Input<string>;
    /**
     * The NameNode that manages the HDFS namespace. The NameNode performs operations such as opening, closing, and renaming files and directories. The NameNode contains the information to map blocks of data to the DataNodes. You can use only one NameNode. See configuration below.
     */
    nameNodes: pulumi.Input<pulumi.Input<inputs.datasync.LocationHdfsNameNode>[]>;
    /**
     * The Quality of Protection (QOP) configuration specifies the Remote Procedure Call (RPC) and data transfer protection settings configured on the Hadoop Distributed File System (HDFS) cluster. If `qopConfiguration` isn't specified, `rpcProtection` and `dataTransferProtection` default to `PRIVACY`. If you set RpcProtection or DataTransferProtection, the other parameter assumes the same value.  See configuration below.
     */
    qopConfiguration?: pulumi.Input<inputs.datasync.LocationHdfsQopConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The number of DataNodes to replicate the data to when writing to the HDFS cluster. By default, data is replicated to three DataNodes.
     */
    replicationFactor?: pulumi.Input<number>;
    /**
     * The user name used to identify the client on the host operating system. If `SIMPLE` is specified for `authenticationType`, this parameter is required.
     */
    simpleUser?: pulumi.Input<string>;
    /**
     * A subdirectory in the HDFS cluster. This subdirectory is used to read data from or write data to the HDFS cluster. If the subdirectory isn't specified, it will default to /.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
