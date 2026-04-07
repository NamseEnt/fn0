import * as pulumi from "@pulumi/pulumi";
/**
 * Provides redshift serverless temporary credentials for a workgroup.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshiftserverless.getCredentials({
 *     workgroupName: exampleAwsRedshiftserverlessWorkgroup.workgroupName,
 * });
 * ```
 */
export declare function getCredentials(args: GetCredentialsArgs, opts?: pulumi.InvokeOptions): Promise<GetCredentialsResult>;
/**
 * A collection of arguments for invoking getCredentials.
 */
export interface GetCredentialsArgs {
    /**
     * The name of the database to get temporary authorization to log on to.
     */
    dbName?: string;
    /**
     * The number of seconds until the returned temporary password expires. The minimum is 900 seconds, and the maximum is 3600 seconds.
     */
    durationSeconds?: number;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * The name of the workgroup associated with the database.
     */
    workgroupName: string;
}
/**
 * A collection of values returned by getCredentials.
 */
export interface GetCredentialsResult {
    readonly dbName?: string;
    /**
     * Temporary password that authorizes the user name returned by `dbUser` to log on to the database `dbName`.
     */
    readonly dbPassword: string;
    /**
     * A database user name that is authorized to log on to the database `dbName` using the password `dbPassword` . If the specified `dbUser` exists in the database, the new user name has the same database privileges as the user named in `dbUser` . By default, the user is added to PUBLIC. the user doesn't exist in the database.
     */
    readonly dbUser: string;
    readonly durationSeconds?: number;
    /**
     * Date and time the password in `dbPassword` expires.
     */
    readonly expiration: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly workgroupName: string;
}
/**
 * Provides redshift serverless temporary credentials for a workgroup.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.redshiftserverless.getCredentials({
 *     workgroupName: exampleAwsRedshiftserverlessWorkgroup.workgroupName,
 * });
 * ```
 */
export declare function getCredentialsOutput(args: GetCredentialsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCredentialsResult>;
/**
 * A collection of arguments for invoking getCredentials.
 */
export interface GetCredentialsOutputArgs {
    /**
     * The name of the database to get temporary authorization to log on to.
     */
    dbName?: pulumi.Input<string>;
    /**
     * The number of seconds until the returned temporary password expires. The minimum is 900 seconds, and the maximum is 3600 seconds.
     */
    durationSeconds?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the workgroup associated with the database.
     */
    workgroupName: pulumi.Input<string>;
}
