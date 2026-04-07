import * as pulumi from "@pulumi/pulumi";
/**
 * Registers a Lake Formation resource (e.g., S3 bucket) as managed by the Data Catalog. In other words, the S3 path is added to the data lake.
 *
 * Choose a role that has read/write access to the chosen Amazon S3 path or use the service-linked role.
 * When you register the S3 path, the service-linked role and a new inline policy are created on your behalf.
 * Lake Formation adds the first path to the inline policy and attaches it to the service-linked role.
 * When you register subsequent paths, Lake Formation adds the path to the existing policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.s3.getBucket({
 *     bucket: "an-example-bucket",
 * });
 * const exampleResource = new aws.lakeformation.Resource("example", {arn: example.then(example => example.arn)});
 * ```
 */
export declare class Resource extends pulumi.CustomResource {
    /**
     * Get an existing Resource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceState, opts?: pulumi.CustomResourceOptions): Resource;
    /**
     * Returns true if the given object is an instance of Resource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Resource;
    /**
     * Amazon Resource Name (ARN) of the resource.
     *
     * The following arguments are optional:
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Flag to enable AWS LakeFormation hybrid access permission mode.
     */
    readonly hybridAccessEnabled: pulumi.Output<boolean>;
    /**
     * Date and time the resource was last modified in [RFC 3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    readonly lastModified: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Role that has read/write access to the resource.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Designates an AWS Identity and Access Management (IAM) service-linked role by registering this role with the Data Catalog.
     */
    readonly useServiceLinkedRole: pulumi.Output<boolean | undefined>;
    /**
     * Whether or not the resource is a federated resource. Set to true when registering AWS Glue connections for federated catalog functionality.
     */
    readonly withFederation: pulumi.Output<boolean>;
    /**
     * Boolean to grant the calling principal the permissions to perform all supported Lake Formation operations on the registered data location.
     *
     * > **NOTE:** AWS does not support registering an S3 location with an IAM role and subsequently updating the S3 location registration to a service-linked role.
     */
    readonly withPrivilegedAccess: pulumi.Output<boolean>;
    /**
     * Create a Resource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Resource resources.
 */
export interface ResourceState {
    /**
     * Amazon Resource Name (ARN) of the resource.
     *
     * The following arguments are optional:
     */
    arn?: pulumi.Input<string>;
    /**
     * Flag to enable AWS LakeFormation hybrid access permission mode.
     */
    hybridAccessEnabled?: pulumi.Input<boolean>;
    /**
     * Date and time the resource was last modified in [RFC 3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    lastModified?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Role that has read/write access to the resource.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Designates an AWS Identity and Access Management (IAM) service-linked role by registering this role with the Data Catalog.
     */
    useServiceLinkedRole?: pulumi.Input<boolean>;
    /**
     * Whether or not the resource is a federated resource. Set to true when registering AWS Glue connections for federated catalog functionality.
     */
    withFederation?: pulumi.Input<boolean>;
    /**
     * Boolean to grant the calling principal the permissions to perform all supported Lake Formation operations on the registered data location.
     *
     * > **NOTE:** AWS does not support registering an S3 location with an IAM role and subsequently updating the S3 location registration to a service-linked role.
     */
    withPrivilegedAccess?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a Resource resource.
 */
export interface ResourceArgs {
    /**
     * Amazon Resource Name (ARN) of the resource.
     *
     * The following arguments are optional:
     */
    arn: pulumi.Input<string>;
    /**
     * Flag to enable AWS LakeFormation hybrid access permission mode.
     */
    hybridAccessEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Role that has read/write access to the resource.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Designates an AWS Identity and Access Management (IAM) service-linked role by registering this role with the Data Catalog.
     */
    useServiceLinkedRole?: pulumi.Input<boolean>;
    /**
     * Whether or not the resource is a federated resource. Set to true when registering AWS Glue connections for federated catalog functionality.
     */
    withFederation?: pulumi.Input<boolean>;
    /**
     * Boolean to grant the calling principal the permissions to perform all supported Lake Formation operations on the registered data location.
     *
     * > **NOTE:** AWS does not support registering an S3 location with an IAM role and subsequently updating the S3 location registration to a service-linked role.
     */
    withPrivilegedAccess?: pulumi.Input<boolean>;
}
