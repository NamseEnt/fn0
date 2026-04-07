import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a QuickSight Folder.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.Folder("example", {
 *     folderId: "example-id",
 *     name: "example-name",
 * });
 * ```
 *
 * ### With Permissions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.Folder("example", {
 *     folderId: "example-id",
 *     name: "example-name",
 *     permissions: [{
 *         actions: [
 *             "quicksight:CreateFolder",
 *             "quicksight:DescribeFolder",
 *             "quicksight:UpdateFolder",
 *             "quicksight:DeleteFolder",
 *             "quicksight:CreateFolderMembership",
 *             "quicksight:DeleteFolderMembership",
 *             "quicksight:DescribeFolderPermissions",
 *             "quicksight:UpdateFolderPermissions",
 *         ],
 *         principal: exampleAwsQuicksightUser.arn,
 *     }],
 * });
 * ```
 *
 * ### With Parent Folder
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const parent = new aws.quicksight.Folder("parent", {
 *     folderId: "parent-id",
 *     name: "parent-name",
 * });
 * const example = new aws.quicksight.Folder("example", {
 *     folderId: "example-id",
 *     name: "example-name",
 *     parentFolderArn: parent.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a QuickSight folder using the AWS account ID and folder ID name separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/folder:Folder example 123456789012,example-id
 * ```
 */
export declare class Folder extends pulumi.CustomResource {
    /**
     * Get an existing Folder resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FolderState, opts?: pulumi.CustomResourceOptions): Folder;
    /**
     * Returns true if the given object is an instance of Folder.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Folder;
    /**
     * ARN of the folder.
     */
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The time that the folder was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Identifier for the folder.
     */
    readonly folderId: pulumi.Output<string>;
    /**
     * An array of ancestor ARN strings for the folder. Empty for root-level folders.
     */
    readonly folderPaths: pulumi.Output<string[]>;
    /**
     * The type of folder. By default, it is `SHARED`. Valid values are: `SHARED`.
     */
    readonly folderType: pulumi.Output<string | undefined>;
    /**
     * The time that the folder was last updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * Display name for the folder.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) for the parent folder. If not set, creates a root-level folder.
     */
    readonly parentFolderArn: pulumi.Output<string | undefined>;
    /**
     * A set of resource permissions on the folder. Maximum of 64 items. See permissions.
     */
    readonly permissions: pulumi.Output<outputs.quicksight.FolderPermission[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Folder resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FolderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Folder resources.
 */
export interface FolderState {
    /**
     * ARN of the folder.
     */
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * The time that the folder was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Identifier for the folder.
     */
    folderId?: pulumi.Input<string>;
    /**
     * An array of ancestor ARN strings for the folder. Empty for root-level folders.
     */
    folderPaths?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of folder. By default, it is `SHARED`. Valid values are: `SHARED`.
     */
    folderType?: pulumi.Input<string>;
    /**
     * The time that the folder was last updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * Display name for the folder.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the parent folder. If not set, creates a root-level folder.
     */
    parentFolderArn?: pulumi.Input<string>;
    /**
     * A set of resource permissions on the folder. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.FolderPermission>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Folder resource.
 */
export interface FolderArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Identifier for the folder.
     */
    folderId: pulumi.Input<string>;
    /**
     * The type of folder. By default, it is `SHARED`. Valid values are: `SHARED`.
     */
    folderType?: pulumi.Input<string>;
    /**
     * Display name for the folder.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the parent folder. If not set, creates a root-level folder.
     */
    parentFolderArn?: pulumi.Input<string>;
    /**
     * A set of resource permissions on the folder. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.FolderPermission>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
