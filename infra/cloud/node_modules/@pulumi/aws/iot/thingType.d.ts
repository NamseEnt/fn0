import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates and manages an AWS IoT Thing Type.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.iot.ThingType("foo", {name: "my_iot_thing"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IOT Thing Types using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/thingType:ThingType example example
 * ```
 */
export declare class ThingType extends pulumi.CustomResource {
    /**
     * Get an existing ThingType resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ThingTypeState, opts?: pulumi.CustomResourceOptions): ThingType;
    /**
     * Returns true if the given object is an instance of ThingType.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ThingType;
    /**
     * The ARN of the created AWS IoT Thing Type.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether the thing type is deprecated. If true, no new things could be associated with this type.
     */
    readonly deprecated: pulumi.Output<boolean | undefined>;
    /**
     * The name of the thing type.
     */
    readonly name: pulumi.Output<string>;
    /**
     * , Configuration block that can contain the following properties of the thing type:
     */
    readonly properties: pulumi.Output<outputs.iot.ThingTypeProperties | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Create a ThingType resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ThingTypeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ThingType resources.
 */
export interface ThingTypeState {
    /**
     * The ARN of the created AWS IoT Thing Type.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether the thing type is deprecated. If true, no new things could be associated with this type.
     */
    deprecated?: pulumi.Input<boolean>;
    /**
     * The name of the thing type.
     */
    name?: pulumi.Input<string>;
    /**
     * , Configuration block that can contain the following properties of the thing type:
     */
    properties?: pulumi.Input<inputs.iot.ThingTypeProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
 * The set of arguments for constructing a ThingType resource.
 */
export interface ThingTypeArgs {
    /**
     * Whether the thing type is deprecated. If true, no new things could be associated with this type.
     */
    deprecated?: pulumi.Input<boolean>;
    /**
     * The name of the thing type.
     */
    name?: pulumi.Input<string>;
    /**
     * , Configuration block that can contain the following properties of the thing type:
     */
    properties?: pulumi.Input<inputs.iot.ThingTypeProperties>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
