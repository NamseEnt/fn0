import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * List the event categories of all the RDS resources.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const example = await aws.rds.getEventCategories({});
 *     return {
 *         example: example.eventCategories,
 *     };
 * }
 * ```
 *
 * List the event categories specific to the RDS resource `db-snapshot`.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const example = await aws.rds.getEventCategories({
 *         sourceType: "db-snapshot",
 *     });
 *     return {
 *         example: example.eventCategories,
 *     };
 * }
 * ```
 */
export declare function getEventCategories(args?: GetEventCategoriesArgs, opts?: pulumi.InvokeOptions): Promise<GetEventCategoriesResult>;
/**
 * A collection of arguments for invoking getEventCategories.
 */
export interface GetEventCategoriesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Type of source that will be generating the events. Valid options are db-instance, db-security-group, db-parameter-group, db-snapshot, db-cluster or db-cluster-snapshot.
     */
    sourceType?: string;
}
/**
 * A collection of values returned by getEventCategories.
 */
export interface GetEventCategoriesResult {
    /**
     * List of the event categories.
     */
    readonly eventCategories: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly sourceType?: string;
}
/**
 * ## Example Usage
 *
 * List the event categories of all the RDS resources.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const example = await aws.rds.getEventCategories({});
 *     return {
 *         example: example.eventCategories,
 *     };
 * }
 * ```
 *
 * List the event categories specific to the RDS resource `db-snapshot`.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const example = await aws.rds.getEventCategories({
 *         sourceType: "db-snapshot",
 *     });
 *     return {
 *         example: example.eventCategories,
 *     };
 * }
 * ```
 */
export declare function getEventCategoriesOutput(args?: GetEventCategoriesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEventCategoriesResult>;
/**
 * A collection of arguments for invoking getEventCategories.
 */
export interface GetEventCategoriesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of source that will be generating the events. Valid options are db-instance, db-security-group, db-parameter-group, db-snapshot, db-cluster or db-cluster-snapshot.
     */
    sourceType?: pulumi.Input<string>;
}
