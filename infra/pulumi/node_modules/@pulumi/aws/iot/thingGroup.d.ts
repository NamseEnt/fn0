import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS IoT Thing Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const parent = new aws.iot.ThingGroup("parent", {name: "parent"});
 * const example = new aws.iot.ThingGroup("example", {
 *     name: "example",
 *     parentGroupName: parent.name,
 *     properties: {
 *         attributePayload: {
 *             attributes: {
 *                 One: "11111",
 *                 Two: "TwoTwo",
 *             },
 *         },
 *         description: "This is my thing group",
 *     },
 *     tags: {
 *         managed: "true",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IoT Things Groups using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/thingGroup:ThingGroup example example
 * ```
 */
export declare class ThingGroup extends pulumi.CustomResource {
    /**
     * Get an existing ThingGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ThingGroupState, opts?: pulumi.CustomResourceOptions): ThingGroup;
    /**
     * Returns true if the given object is an instance of ThingGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ThingGroup;
    /**
     * The ARN of the Thing Group.
     */
    readonly arn: pulumi.Output<string>;
    readonly metadatas: pulumi.Output<outputs.iot.ThingGroupMetadata[]>;
    /**
     * The name of the Thing Group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The name of the parent Thing Group.
     */
    readonly parentGroupName: pulumi.Output<string | undefined>;
    /**
     * The Thing Group properties. Defined below.
     */
    readonly properties: pulumi.Output<outputs.iot.ThingGroupProperties | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The current version of the Thing Group record in the registry.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a ThingGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ThingGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ThingGroup resources.
 */
export interface ThingGroupState {
    /**
     * The ARN of the Thing Group.
     */
    arn?: pulumi.Input<string>;
    metadatas?: pulumi.Input<pulumi.Input<inputs.iot.ThingGroupMetadata>[]>;
    /**
     * The name of the Thing Group.
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the parent Thing Group.
     */
    parentGroupName?: pulumi.Input<string>;
    /**
     * The Thing Group properties. Defined below.
     */
    properties?: pulumi.Input<inputs.iot.ThingGroupProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The current version of the Thing Group record in the registry.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a ThingGroup resource.
 */
export interface ThingGroupArgs {
    /**
     * The name of the Thing Group.
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the parent Thing Group.
     */
    parentGroupName?: pulumi.Input<string>;
    /**
     * The Thing Group properties. Defined below.
     */
    properties?: pulumi.Input<inputs.iot.ThingGroupProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
