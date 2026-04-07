import * as pulumi from "@pulumi/pulumi";
/**
 * This data source can be used to fetch information about all
 * IAM instance profiles under a role. By using this data source, you can reference IAM
 * instance profile properties without having to hard code ARNs as input.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getInstanceProfiles({
 *     roleName: "an_example_iam_role_name",
 * });
 * ```
 */
export declare function getInstanceProfiles(args: GetInstanceProfilesArgs, opts?: pulumi.InvokeOptions): Promise<GetInstanceProfilesResult>;
/**
 * A collection of arguments for invoking getInstanceProfiles.
 */
export interface GetInstanceProfilesArgs {
    /**
     * IAM role name.
     */
    roleName: string;
}
/**
 * A collection of values returned by getInstanceProfiles.
 */
export interface GetInstanceProfilesResult {
    /**
     * Set of ARNs of instance profiles.
     */
    readonly arns: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of IAM instance profile names.
     */
    readonly names: string[];
    /**
     * Set of IAM instance profile paths.
     */
    readonly paths: string[];
    readonly roleName: string;
}
/**
 * This data source can be used to fetch information about all
 * IAM instance profiles under a role. By using this data source, you can reference IAM
 * instance profile properties without having to hard code ARNs as input.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getInstanceProfiles({
 *     roleName: "an_example_iam_role_name",
 * });
 * ```
 */
export declare function getInstanceProfilesOutput(args: GetInstanceProfilesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetInstanceProfilesResult>;
/**
 * A collection of arguments for invoking getInstanceProfiles.
 */
export interface GetInstanceProfilesOutputArgs {
    /**
     * IAM role name.
     */
    roleName: pulumi.Input<string>;
}
