import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Service Catalog AppRegistry Attribute Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getAppregistryAttributeGroup({
 *     name: "example_attribute_group",
 * });
 * ```
 */
export declare function getAppregistryAttributeGroup(args?: GetAppregistryAttributeGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetAppregistryAttributeGroupResult>;
/**
 * A collection of arguments for invoking getAppregistryAttributeGroup.
 */
export interface GetAppregistryAttributeGroupArgs {
    /**
     * ARN of the Attribute Group to find.
     */
    arn?: string;
    /**
     * ID of the Attribute Group to find.
     */
    id?: string;
    /**
     * Name of the Attribute Group to find.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * > Exactly one of `arn`, `id`, or `name` must be set.
     */
    region?: string;
}
/**
 * A collection of values returned by getAppregistryAttributeGroup.
 */
export interface GetAppregistryAttributeGroupResult {
    readonly arn: string;
    /**
     * A JSON string of nested key-value pairs that represents the attributes of the group.
     */
    readonly attributes: string;
    /**
     * Description of the Attribute Group.
     */
    readonly description: string;
    readonly id: string;
    readonly name: string;
    readonly region: string;
    /**
     * A map of tags assigned to the Attribute Group. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS Service Catalog AppRegistry Attribute Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getAppregistryAttributeGroup({
 *     name: "example_attribute_group",
 * });
 * ```
 */
export declare function getAppregistryAttributeGroupOutput(args?: GetAppregistryAttributeGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAppregistryAttributeGroupResult>;
/**
 * A collection of arguments for invoking getAppregistryAttributeGroup.
 */
export interface GetAppregistryAttributeGroupOutputArgs {
    /**
     * ARN of the Attribute Group to find.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the Attribute Group to find.
     */
    id?: pulumi.Input<string>;
    /**
     * Name of the Attribute Group to find.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * > Exactly one of `arn`, `id`, or `name` must be set.
     */
    region?: pulumi.Input<string>;
}
