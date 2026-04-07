import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic File System (EFS) access point.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.efs.AccessPoint("test", {fileSystemId: foo.id});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the EFS access points using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:efs/accessPoint:AccessPoint test fsap-52a643fb
 * ```
 */
export declare class AccessPoint extends pulumi.CustomResource {
    /**
     * Get an existing AccessPoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccessPointState, opts?: pulumi.CustomResourceOptions): AccessPoint;
    /**
     * Returns true if the given object is an instance of AccessPoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccessPoint;
    /**
     * ARN of the access point.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of the file system.
     */
    readonly fileSystemArn: pulumi.Output<string>;
    /**
     * ID of the file system for which the access point is intended.
     */
    readonly fileSystemId: pulumi.Output<string>;
    readonly ownerId: pulumi.Output<string>;
    /**
     * Operating system user and group applied to all file system requests made using the access point. Detailed below.
     */
    readonly posixUser: pulumi.Output<outputs.efs.AccessPointPosixUser | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Directory on the Amazon EFS file system that the access point provides access to. Detailed below.
     */
    readonly rootDirectory: pulumi.Output<outputs.efs.AccessPointRootDirectory>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a AccessPoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccessPointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccessPoint resources.
 */
export interface AccessPointState {
    /**
     * ARN of the access point.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the file system.
     */
    fileSystemArn?: pulumi.Input<string>;
    /**
     * ID of the file system for which the access point is intended.
     */
    fileSystemId?: pulumi.Input<string>;
    ownerId?: pulumi.Input<string>;
    /**
     * Operating system user and group applied to all file system requests made using the access point. Detailed below.
     */
    posixUser?: pulumi.Input<inputs.efs.AccessPointPosixUser>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Directory on the Amazon EFS file system that the access point provides access to. Detailed below.
     */
    rootDirectory?: pulumi.Input<inputs.efs.AccessPointRootDirectory>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a AccessPoint resource.
 */
export interface AccessPointArgs {
    /**
     * ID of the file system for which the access point is intended.
     */
    fileSystemId: pulumi.Input<string>;
    /**
     * Operating system user and group applied to all file system requests made using the access point. Detailed below.
     */
    posixUser?: pulumi.Input<inputs.efs.AccessPointPosixUser>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Directory on the Amazon EFS file system that the access point provides access to. Detailed below.
     */
    rootDirectory?: pulumi.Input<inputs.efs.AccessPointRootDirectory>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
