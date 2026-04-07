import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the default tags configured on the provider.
 *
 * With this data source, you can apply default tags to resources not _directly_ managed by a resource, such as the instances underneath an Auto Scaling group or the volumes created for an EC2 instance.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.getDefaultTags({});
 * ```
 *
 * ### Dynamically Apply Default Tags to Auto Scaling Group
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.getDefaultTags({});
 * const exampleGroup = new aws.autoscaling.Group("example", {tags: .map(entry => ({
 *     key: entry.key,
 *     value: entry.value,
 *     propagateAtLaunch: true,
 * }))});
 * ```
 */
export declare function getDefaultTags(args?: GetDefaultTagsArgs, opts?: pulumi.InvokeOptions): Promise<GetDefaultTagsResult>;
/**
 * A collection of arguments for invoking getDefaultTags.
 */
export interface GetDefaultTagsArgs {
    id?: string;
}
/**
 * A collection of values returned by getDefaultTags.
 */
export interface GetDefaultTagsResult {
    readonly id: string;
    /**
     * Key-value mapping of provider default tags.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Use this data source to get the default tags configured on the provider.
 *
 * With this data source, you can apply default tags to resources not _directly_ managed by a resource, such as the instances underneath an Auto Scaling group or the volumes created for an EC2 instance.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.getDefaultTags({});
 * ```
 *
 * ### Dynamically Apply Default Tags to Auto Scaling Group
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.getDefaultTags({});
 * const exampleGroup = new aws.autoscaling.Group("example", {tags: .map(entry => ({
 *     key: entry.key,
 *     value: entry.value,
 *     propagateAtLaunch: true,
 * }))});
 * ```
 */
export declare function getDefaultTagsOutput(args?: GetDefaultTagsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDefaultTagsResult>;
/**
 * A collection of arguments for invoking getDefaultTags.
 */
export interface GetDefaultTagsOutputArgs {
    id?: pulumi.Input<string>;
}
