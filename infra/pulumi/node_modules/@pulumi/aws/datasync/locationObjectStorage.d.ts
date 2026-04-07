import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Object Storage Location within AWS DataSync.
 *
 * > **NOTE:** The DataSync Agents must be available before creating this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.LocationObjectStorage("example", {
 *     agentArns: [exampleAwsDatasyncAgent.arn],
 *     serverHostname: "example",
 *     bucketName: "example",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DataSync object storage location.
 *
 * Using `pulumi import`, import `aws_datasync_location_object_storage` using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_datasync_location_object_storage.example arn:aws:datasync:us-east-1:123456789012:location/loc-12345678901234567
 */
export declare class LocationObjectStorage extends pulumi.CustomResource {
    /**
     * Get an existing LocationObjectStorage resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LocationObjectStorageState, opts?: pulumi.CustomResourceOptions): LocationObjectStorage;
    /**
     * Returns true if the given object is an instance of LocationObjectStorage.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LocationObjectStorage;
    /**
     * The access key is used if credentials are required to access the self-managed object storage server. If your object storage requires a user name and password to authenticate, use `accessKey` and `secretKey` to provide the user name and password, respectively.
     */
    readonly accessKey: pulumi.Output<string | undefined>;
    /**
     * A list of DataSync Agent ARNs with which this location will be associated. For agentless cross-cloud transfers, this parameter does not need to be specified.
     */
    readonly agentArns: pulumi.Output<string[] | undefined>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The bucket on the self-managed object storage server that is used to read data from.
     */
    readonly bucketName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The secret key is used if credentials are required to access the self-managed object storage server. If your object storage requires a user name and password to authenticate, use `accessKey` and `secretKey` to provide the user name and password, respectively.
     */
    readonly secretKey: pulumi.Output<string | undefined>;
    /**
     * Specifies a certificate to authenticate with an object storage system that uses a private or self-signed certificate authority (CA). You must specify a Base64-encoded .pem string. The certificate can be up to 32768 bytes (before Base64 encoding).
     */
    readonly serverCertificate: pulumi.Output<string | undefined>;
    /**
     * The name of the self-managed object storage server. This value is the IP address or Domain Name Service (DNS) name of the object storage server. An agent uses this host name to mount the object storage server in a network.
     */
    readonly serverHostname: pulumi.Output<string>;
    /**
     * The port that your self-managed object storage server accepts inbound network traffic on. The server port is set by default to TCP 80 (`HTTP`) or TCP 443 (`HTTPS`). You can specify a custom port if your self-managed object storage server requires one.
     */
    readonly serverPort: pulumi.Output<number | undefined>;
    /**
     * The protocol that the object storage server uses to communicate. Valid values are `HTTP` or `HTTPS`.
     */
    readonly serverProtocol: pulumi.Output<string | undefined>;
    /**
     * A subdirectory in the HDFS cluster. This subdirectory is used to read data from or write data to the HDFS cluster. If the subdirectory isn't specified, it will default to /.
     */
    readonly subdirectory: pulumi.Output<string>;
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
    /**
     * The URL of the Object Storage location that was described.
     */
    readonly uri: pulumi.Output<string>;
    /**
     * Create a LocationObjectStorage resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LocationObjectStorageArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LocationObjectStorage resources.
 */
export interface LocationObjectStorageState {
    /**
     * The access key is used if credentials are required to access the self-managed object storage server. If your object storage requires a user name and password to authenticate, use `accessKey` and `secretKey` to provide the user name and password, respectively.
     */
    accessKey?: pulumi.Input<string>;
    /**
     * A list of DataSync Agent ARNs with which this location will be associated. For agentless cross-cloud transfers, this parameter does not need to be specified.
     */
    agentArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    arn?: pulumi.Input<string>;
    /**
     * The bucket on the self-managed object storage server that is used to read data from.
     */
    bucketName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The secret key is used if credentials are required to access the self-managed object storage server. If your object storage requires a user name and password to authenticate, use `accessKey` and `secretKey` to provide the user name and password, respectively.
     */
    secretKey?: pulumi.Input<string>;
    /**
     * Specifies a certificate to authenticate with an object storage system that uses a private or self-signed certificate authority (CA). You must specify a Base64-encoded .pem string. The certificate can be up to 32768 bytes (before Base64 encoding).
     */
    serverCertificate?: pulumi.Input<string>;
    /**
     * The name of the self-managed object storage server. This value is the IP address or Domain Name Service (DNS) name of the object storage server. An agent uses this host name to mount the object storage server in a network.
     */
    serverHostname?: pulumi.Input<string>;
    /**
     * The port that your self-managed object storage server accepts inbound network traffic on. The server port is set by default to TCP 80 (`HTTP`) or TCP 443 (`HTTPS`). You can specify a custom port if your self-managed object storage server requires one.
     */
    serverPort?: pulumi.Input<number>;
    /**
     * The protocol that the object storage server uses to communicate. Valid values are `HTTP` or `HTTPS`.
     */
    serverProtocol?: pulumi.Input<string>;
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
    /**
     * The URL of the Object Storage location that was described.
     */
    uri?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LocationObjectStorage resource.
 */
export interface LocationObjectStorageArgs {
    /**
     * The access key is used if credentials are required to access the self-managed object storage server. If your object storage requires a user name and password to authenticate, use `accessKey` and `secretKey` to provide the user name and password, respectively.
     */
    accessKey?: pulumi.Input<string>;
    /**
     * A list of DataSync Agent ARNs with which this location will be associated. For agentless cross-cloud transfers, this parameter does not need to be specified.
     */
    agentArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The bucket on the self-managed object storage server that is used to read data from.
     */
    bucketName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The secret key is used if credentials are required to access the self-managed object storage server. If your object storage requires a user name and password to authenticate, use `accessKey` and `secretKey` to provide the user name and password, respectively.
     */
    secretKey?: pulumi.Input<string>;
    /**
     * Specifies a certificate to authenticate with an object storage system that uses a private or self-signed certificate authority (CA). You must specify a Base64-encoded .pem string. The certificate can be up to 32768 bytes (before Base64 encoding).
     */
    serverCertificate?: pulumi.Input<string>;
    /**
     * The name of the self-managed object storage server. This value is the IP address or Domain Name Service (DNS) name of the object storage server. An agent uses this host name to mount the object storage server in a network.
     */
    serverHostname: pulumi.Input<string>;
    /**
     * The port that your self-managed object storage server accepts inbound network traffic on. The server port is set by default to TCP 80 (`HTTP`) or TCP 443 (`HTTPS`). You can specify a custom port if your self-managed object storage server requires one.
     */
    serverPort?: pulumi.Input<number>;
    /**
     * The protocol that the object storage server uses to communicate. Valid values are `HTTP` or `HTTPS`.
     */
    serverProtocol?: pulumi.Input<string>;
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
