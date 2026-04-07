import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Resource Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.resourcegroups.Group("test", {
 *     name: "test-group",
 *     resourceQuery: {
 *         query: `{
 *   \\"ResourceTypeFilters\\": [
 *     \\"AWS::EC2::Instance\\"
 *   ],
 *   \\"TagFilters\\": [
 *     {
 *       \\"Key\\": \\"Stage\\",
 *       \\"Values\\": [\\"Test\\"]
 *     }
 *   ]
 * }
 * `,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import resource groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:resourcegroups/group:Group foo resource-group-name
 * ```
 */
export declare class Group extends pulumi.CustomResource {
    /**
     * Get an existing Group resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GroupState, opts?: pulumi.CustomResourceOptions): Group;
    /**
     * Returns true if the given object is an instance of Group.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Group;
    /**
     * The ARN assigned by AWS for this resource group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A configuration associates the resource group with an AWS service and specifies how the service can interact with the resources in the group. See below for details.
     */
    readonly configurations: pulumi.Output<outputs.resourcegroups.GroupConfiguration[] | undefined>;
    /**
     * A description of the resource group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The resource group's name. A resource group name can have a maximum of 127 characters, including letters, numbers, hyphens, dots, and underscores. The name cannot start with `AWS` or `aws`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A `resourceQuery` block. Resource queries are documented below.
     */
    readonly resourceQuery: pulumi.Output<outputs.resourcegroups.GroupResourceQuery | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Group resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: GroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Group resources.
 */
export interface GroupState {
    /**
     * The ARN assigned by AWS for this resource group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A configuration associates the resource group with an AWS service and specifies how the service can interact with the resources in the group. See below for details.
     */
    configurations?: pulumi.Input<pulumi.Input<inputs.resourcegroups.GroupConfiguration>[]>;
    /**
     * A description of the resource group.
     */
    description?: pulumi.Input<string>;
    /**
     * The resource group's name. A resource group name can have a maximum of 127 characters, including letters, numbers, hyphens, dots, and underscores. The name cannot start with `AWS` or `aws`.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A `resourceQuery` block. Resource queries are documented below.
     */
    resourceQuery?: pulumi.Input<inputs.resourcegroups.GroupResourceQuery>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Group resource.
 */
export interface GroupArgs {
    /**
     * A configuration associates the resource group with an AWS service and specifies how the service can interact with the resources in the group. See below for details.
     */
    configurations?: pulumi.Input<pulumi.Input<inputs.resourcegroups.GroupConfiguration>[]>;
    /**
     * A description of the resource group.
     */
    description?: pulumi.Input<string>;
    /**
     * The resource group's name. A resource group name can have a maximum of 127 characters, including letters, numbers, hyphens, dots, and underscores. The name cannot start with `AWS` or `aws`.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A `resourceQuery` block. Resource queries are documented below.
     */
    resourceQuery?: pulumi.Input<inputs.resourcegroups.GroupResourceQuery>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
