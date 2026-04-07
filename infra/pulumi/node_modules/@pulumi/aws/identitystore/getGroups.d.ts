import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS SSO Identity Store Groups.
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
 * const exampleGetGroups = example.then(example => aws.identitystore.getGroups({
 *     identityStoreId: example.identityStoreIds?.[0],
 * }));
 * ```
 */
export declare function getGroups(args: GetGroupsArgs, opts?: pulumi.InvokeOptions): Promise<GetGroupsResult>;
/**
 * A collection of arguments for invoking getGroups.
 */
export interface GetGroupsArgs {
    /**
     * Identity Store ID associated with the Single Sign-On (SSO) Instance.
     */
    identityStoreId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getGroups.
 */
export interface GetGroupsResult {
    /**
     * List of Identity Store Groups
     */
    readonly groups: outputs.identitystore.GetGroupsGroup[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly identityStoreId: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS SSO Identity Store Groups.
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
 * const exampleGetGroups = example.then(example => aws.identitystore.getGroups({
 *     identityStoreId: example.identityStoreIds?.[0],
 * }));
 * ```
 */
export declare function getGroupsOutput(args: GetGroupsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGroupsResult>;
/**
 * A collection of arguments for invoking getGroups.
 */
export interface GetGroupsOutputArgs {
    /**
     * Identity Store ID associated with the Single Sign-On (SSO) Instance.
     */
    identityStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
