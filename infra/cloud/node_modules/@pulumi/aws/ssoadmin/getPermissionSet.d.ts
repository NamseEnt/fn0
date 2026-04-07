import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get a Single Sign-On (SSO) Permission Set.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleGetPermissionSet = example.then(example => aws.ssoadmin.getPermissionSet({
 *     instanceArn: example.arns?.[0],
 *     name: "Example",
 * }));
 * export const arn = exampleGetPermissionSet.then(exampleGetPermissionSet => exampleGetPermissionSet.arn);
 * ```
 */
export declare function getPermissionSet(args: GetPermissionSetArgs, opts?: pulumi.InvokeOptions): Promise<GetPermissionSetResult>;
/**
 * A collection of arguments for invoking getPermissionSet.
 */
export interface GetPermissionSetArgs {
    /**
     * ARN of the permission set.
     */
    arn?: string;
    /**
     * ARN of the SSO Instance associated with the permission set.
     */
    instanceArn: string;
    /**
     * Name of the SSO Permission Set.
     *
     * > **NOTE:** Either `arn` or `name` must be configured.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getPermissionSet.
 */
export interface GetPermissionSetResult {
    readonly arn: string;
    readonly createdDate: string;
    /**
     * Description of the Permission Set.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceArn: string;
    readonly name: string;
    readonly region: string;
    /**
     * Relay state URL used to redirect users within the application during the federation authentication process.
     */
    readonly relayState: string;
    /**
     * Length of time that the application user sessions are valid in the ISO-8601 standard.
     */
    readonly sessionDuration: string;
    /**
     * Key-value map of resource tags.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Use this data source to get a Single Sign-On (SSO) Permission Set.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleGetPermissionSet = example.then(example => aws.ssoadmin.getPermissionSet({
 *     instanceArn: example.arns?.[0],
 *     name: "Example",
 * }));
 * export const arn = exampleGetPermissionSet.then(exampleGetPermissionSet => exampleGetPermissionSet.arn);
 * ```
 */
export declare function getPermissionSetOutput(args: GetPermissionSetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPermissionSetResult>;
/**
 * A collection of arguments for invoking getPermissionSet.
 */
export interface GetPermissionSetOutputArgs {
    /**
     * ARN of the permission set.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the SSO Instance associated with the permission set.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * Name of the SSO Permission Set.
     *
     * > **NOTE:** Either `arn` or `name` must be configured.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
