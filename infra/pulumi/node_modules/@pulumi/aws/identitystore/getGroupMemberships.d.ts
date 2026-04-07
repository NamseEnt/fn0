import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Use this data source to get a list of members in an Identity Store Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleGetGroup = example.then(example => aws.identitystore.getGroup({
 *     identityStoreId: example.identityStoreIds?.[0],
 *     alternateIdentifier: {
 *         uniqueAttribute: {
 *             attributePath: "DisplayName",
 *             attributeValue: "ExampleGroup",
 *         },
 *     },
 * }));
 * const exampleGetGroupMemberships = Promise.all([example, exampleGetGroup]).then(([example, exampleGetGroup]) => aws.identitystore.getGroupMemberships({
 *     identityStoreId: example.identityStoreIds?.[0],
 *     groupId: exampleGetGroup.groupId,
 * }));
 * ```
 */
export declare function getGroupMemberships(args: GetGroupMembershipsArgs, opts?: pulumi.InvokeOptions): Promise<GetGroupMembershipsResult>;
/**
 * A collection of arguments for invoking getGroupMemberships.
 */
export interface GetGroupMembershipsArgs {
    /**
     * The identifier for a group in the Identity Store.
     */
    groupId: string;
    /**
     * Identity Store ID associated with the Single Sign-On Instance.
     */
    identityStoreId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getGroupMemberships.
 */
export interface GetGroupMembershipsResult {
    /**
     * Group identifier.
     */
    readonly groupId: string;
    /**
     * A list of group membership objects. See `groupMemberships` below.
     */
    readonly groupMemberships: outputs.identitystore.GetGroupMembershipsGroupMembership[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Identity store identifier.
     */
    readonly identityStoreId: string;
    readonly region: string;
}
/**
 * Use this data source to get a list of members in an Identity Store Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleGetGroup = example.then(example => aws.identitystore.getGroup({
 *     identityStoreId: example.identityStoreIds?.[0],
 *     alternateIdentifier: {
 *         uniqueAttribute: {
 *             attributePath: "DisplayName",
 *             attributeValue: "ExampleGroup",
 *         },
 *     },
 * }));
 * const exampleGetGroupMemberships = Promise.all([example, exampleGetGroup]).then(([example, exampleGetGroup]) => aws.identitystore.getGroupMemberships({
 *     identityStoreId: example.identityStoreIds?.[0],
 *     groupId: exampleGetGroup.groupId,
 * }));
 * ```
 */
export declare function getGroupMembershipsOutput(args: GetGroupMembershipsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGroupMembershipsResult>;
/**
 * A collection of arguments for invoking getGroupMemberships.
 */
export interface GetGroupMembershipsOutputArgs {
    /**
     * The identifier for a group in the Identity Store.
     */
    groupId: pulumi.Input<string>;
    /**
     * Identity Store ID associated with the Single Sign-On Instance.
     */
    identityStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
