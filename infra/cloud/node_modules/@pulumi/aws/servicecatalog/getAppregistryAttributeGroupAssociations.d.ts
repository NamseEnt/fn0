import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing AWS Service Catalog AppRegistry Attribute Group Associations.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getAppregistryAttributeGroupAssociations({
 *     id: "12456778723424sdffsdfsdq34,12234t3564dsfsdf34asff4ww3",
 * });
 * ```
 */
export declare function getAppregistryAttributeGroupAssociations(args?: GetAppregistryAttributeGroupAssociationsArgs, opts?: pulumi.InvokeOptions): Promise<GetAppregistryAttributeGroupAssociationsResult>;
/**
 * A collection of arguments for invoking getAppregistryAttributeGroupAssociations.
 */
export interface GetAppregistryAttributeGroupAssociationsArgs {
    /**
     * ID of the application to which attribute groups are associated.
     */
    id?: string;
    /**
     * Name of the application to which attribute groups are associated.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * > Exactly one of `id`or `name` must be set.
     */
    region?: string;
}
/**
 * A collection of values returned by getAppregistryAttributeGroupAssociations.
 */
export interface GetAppregistryAttributeGroupAssociationsResult {
    /**
     * Set of attribute group IDs this application is associated with.
     */
    readonly attributeGroupIds: string[];
    readonly id?: string;
    readonly name?: string;
    readonly region: string;
}
/**
 * Data source for managing AWS Service Catalog AppRegistry Attribute Group Associations.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getAppregistryAttributeGroupAssociations({
 *     id: "12456778723424sdffsdfsdq34,12234t3564dsfsdf34asff4ww3",
 * });
 * ```
 */
export declare function getAppregistryAttributeGroupAssociationsOutput(args?: GetAppregistryAttributeGroupAssociationsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAppregistryAttributeGroupAssociationsResult>;
/**
 * A collection of arguments for invoking getAppregistryAttributeGroupAssociations.
 */
export interface GetAppregistryAttributeGroupAssociationsOutputArgs {
    /**
     * ID of the application to which attribute groups are associated.
     */
    id?: pulumi.Input<string>;
    /**
     * Name of the application to which attribute groups are associated.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * > Exactly one of `id`or `name` must be set.
     */
    region?: pulumi.Input<string>;
}
