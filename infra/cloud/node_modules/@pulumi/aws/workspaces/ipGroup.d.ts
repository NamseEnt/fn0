import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an IP access control group in AWS WorkSpaces Service
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const contractors = new aws.workspaces.IpGroup("contractors", {
 *     name: "Contractors",
 *     description: "Contractors IP access control group",
 *     rules: [
 *         {
 *             source: "150.24.14.0/24",
 *             description: "NY",
 *         },
 *         {
 *             source: "125.191.14.85/32",
 *             description: "LA",
 *         },
 *         {
 *             source: "44.98.100.0/24",
 *             description: "STL",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces IP groups using their GroupID. For example:
 *
 * ```sh
 * $ pulumi import aws:workspaces/ipGroup:IpGroup example wsipg-488lrtl3k
 * ```
 */
export declare class IpGroup extends pulumi.CustomResource {
    /**
     * Get an existing IpGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IpGroupState, opts?: pulumi.CustomResourceOptions): IpGroup;
    /**
     * Returns true if the given object is an instance of IpGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IpGroup;
    /**
     * The description of the IP group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the IP group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * One or more pairs specifying the IP group rule (in CIDR format) from which web requests originate.
     */
    readonly rules: pulumi.Output<outputs.workspaces.IpGroupRule[] | undefined>;
    /**
     * A map of tags assigned to the WorkSpaces directory. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a IpGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: IpGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IpGroup resources.
 */
export interface IpGroupState {
    /**
     * The description of the IP group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the IP group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more pairs specifying the IP group rule (in CIDR format) from which web requests originate.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.workspaces.IpGroupRule>[]>;
    /**
     * A map of tags assigned to the WorkSpaces directory. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a IpGroup resource.
 */
export interface IpGroupArgs {
    /**
     * The description of the IP group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the IP group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more pairs specifying the IP group rule (in CIDR format) from which web requests originate.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.workspaces.IpGroupRule>[]>;
    /**
     * A map of tags assigned to the WorkSpaces directory. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
