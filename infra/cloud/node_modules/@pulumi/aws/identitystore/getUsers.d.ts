import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Use this data source to get a list of users in an Identity Store instance.
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
 * const exampleGetUsers = example.then(example => aws.identitystore.getUsers({
 *     identityStoreId: example.identityStoreIds?.[0],
 * }));
 * ```
 */
export declare function getUsers(args: GetUsersArgs, opts?: pulumi.InvokeOptions): Promise<GetUsersResult>;
/**
 * A collection of arguments for invoking getUsers.
 */
export interface GetUsersArgs {
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
 * A collection of values returned by getUsers.
 */
export interface GetUsersResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly identityStoreId: string;
    /**
     * Region of the address.
     */
    readonly region: string;
    /**
     * List of Identity Store Users
     */
    readonly users: outputs.identitystore.GetUsersUser[];
}
/**
 * Use this data source to get a list of users in an Identity Store instance.
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
 * const exampleGetUsers = example.then(example => aws.identitystore.getUsers({
 *     identityStoreId: example.identityStoreIds?.[0],
 * }));
 * ```
 */
export declare function getUsersOutput(args: GetUsersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUsersResult>;
/**
 * A collection of arguments for invoking getUsers.
 */
export interface GetUsersOutputArgs {
    /**
     * Identity Store ID associated with the Single Sign-On Instance.
     */
    identityStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
