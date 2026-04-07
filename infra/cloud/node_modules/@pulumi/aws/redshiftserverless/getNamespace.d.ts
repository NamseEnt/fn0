import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Redshift Serverless Namespace.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshiftserverless.getNamespace({
 *     namespaceName: "example-namespace",
 * });
 * ```
 */
export declare function getNamespace(args: GetNamespaceArgs, opts?: pulumi.InvokeOptions): Promise<GetNamespaceResult>;
/**
 * A collection of arguments for invoking getNamespace.
 */
export interface GetNamespaceArgs {
    /**
     * The name of the namespace.
     */
    namespaceName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getNamespace.
 */
export interface GetNamespaceResult {
    /**
     * The username of the administrator for the first database created in the namespace.
     */
    readonly adminUsername: string;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Namespace.
     */
    readonly arn: string;
    /**
     * The name of the first database created in the namespace.
     */
    readonly dbName: string;
    /**
     * The Amazon Resource Name (ARN) of the IAM role to set as a default in the namespace. When specifying `defaultIamRoleArn`, it also must be part of `iamRoles`.
     */
    readonly defaultIamRoleArn: string;
    /**
     * A list of IAM roles to associate with the namespace.
     */
    readonly iamRoles: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The ARN of the Amazon Web Services Key Management Service key used to encrypt your data.
     */
    readonly kmsKeyId: string;
    /**
     * The types of logs the namespace can export. Available export types are `userlog`, `connectionlog`, and `useractivitylog`.
     */
    readonly logExports: string[];
    /**
     * The Redshift Namespace ID.
     */
    readonly namespaceId: string;
    readonly namespaceName: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS Redshift Serverless Namespace.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshiftserverless.getNamespace({
 *     namespaceName: "example-namespace",
 * });
 * ```
 */
export declare function getNamespaceOutput(args: GetNamespaceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetNamespaceResult>;
/**
 * A collection of arguments for invoking getNamespace.
 */
export interface GetNamespaceOutputArgs {
    /**
     * The name of the namespace.
     */
    namespaceName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
