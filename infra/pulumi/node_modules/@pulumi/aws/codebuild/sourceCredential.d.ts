import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeBuild Source Credentials Resource.
 *
 * > **NOTE:
 * ** [Codebuild only allows a single credential per given server type in a given region](https://docs.aws.amazon.com/cdk/api/v2/docs/aws-cdk-lib.aws_codebuild.GitHubSourceCredentials.html).
 * Therefore, when you define `aws.codebuild.SourceCredential`,
 * `aws.codebuild.Project` resource defined in the same module will use it.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codebuild.SourceCredential("example", {
 *     authType: "PERSONAL_ACCESS_TOKEN",
 *     serverType: "GITHUB",
 *     token: "example",
 * });
 * ```
 *
 * ### Bitbucket Server Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codebuild.SourceCredential("example", {
 *     authType: "BASIC_AUTH",
 *     serverType: "BITBUCKET",
 *     token: "example",
 *     userName: "test-user",
 * });
 * ```
 *
 * ### AWS CodeStar Connection Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codebuild.SourceCredential("example", {
 *     authType: "CODECONNECTIONS",
 *     serverType: "GITHUB",
 *     token: "arn:aws:codestar-connections:us-east-1:123456789012:connection/guid-string",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodeBuild source credential.
 *
 * Using `pulumi import`, import CodeBuild Source Credential using the CodeBuild Source Credential arn. For example:
 *
 * console
 *
 * % pulumi import aws_codebuild_source_credential.example arn:aws:codebuild:us-west-2:123456789:token:github
 */
export declare class SourceCredential extends pulumi.CustomResource {
    /**
     * Get an existing SourceCredential resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SourceCredentialState, opts?: pulumi.CustomResourceOptions): SourceCredential;
    /**
     * Returns true if the given object is an instance of SourceCredential.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SourceCredential;
    /**
     * The ARN of Source Credential.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The type of authentication used to connect to a GitHub, GitHub Enterprise, or Bitbucket
     * repository. Valid values are `BASIC_AUTH`,
     * `PERSONAL_ACCESS_TOKEN`, `CODECONNECTIONS`, and `SECRETS_MANAGER`. An OAUTH connection is not supported by the API.
     */
    readonly authType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The source provider used for this project.
     */
    readonly serverType: pulumi.Output<string>;
    /**
     * For a GitHub and GitHub Enterprise, this is the personal access token. For Bitbucket, this is the
     * app password. When using an AWS CodeStar connection (`authType = "CODECONNECTIONS")`, this is an AWS CodeStar
     * Connection ARN.
     */
    readonly token: pulumi.Output<string>;
    /**
     * The Bitbucket username when the authType is `BASIC_AUTH`. This parameter is not valid for
     * other types of source providers or connections.
     */
    readonly userName: pulumi.Output<string | undefined>;
    /**
     * Create a SourceCredential resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SourceCredentialArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SourceCredential resources.
 */
export interface SourceCredentialState {
    /**
     * The ARN of Source Credential.
     */
    arn?: pulumi.Input<string>;
    /**
     * The type of authentication used to connect to a GitHub, GitHub Enterprise, or Bitbucket
     * repository. Valid values are `BASIC_AUTH`,
     * `PERSONAL_ACCESS_TOKEN`, `CODECONNECTIONS`, and `SECRETS_MANAGER`. An OAUTH connection is not supported by the API.
     */
    authType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The source provider used for this project.
     */
    serverType?: pulumi.Input<string>;
    /**
     * For a GitHub and GitHub Enterprise, this is the personal access token. For Bitbucket, this is the
     * app password. When using an AWS CodeStar connection (`authType = "CODECONNECTIONS")`, this is an AWS CodeStar
     * Connection ARN.
     */
    token?: pulumi.Input<string>;
    /**
     * The Bitbucket username when the authType is `BASIC_AUTH`. This parameter is not valid for
     * other types of source providers or connections.
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SourceCredential resource.
 */
export interface SourceCredentialArgs {
    /**
     * The type of authentication used to connect to a GitHub, GitHub Enterprise, or Bitbucket
     * repository. Valid values are `BASIC_AUTH`,
     * `PERSONAL_ACCESS_TOKEN`, `CODECONNECTIONS`, and `SECRETS_MANAGER`. An OAUTH connection is not supported by the API.
     */
    authType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The source provider used for this project.
     */
    serverType: pulumi.Input<string>;
    /**
     * For a GitHub and GitHub Enterprise, this is the personal access token. For Bitbucket, this is the
     * app password. When using an AWS CodeStar connection (`authType = "CODECONNECTIONS")`, this is an AWS CodeStar
     * Connection ARN.
     */
    token: pulumi.Input<string>;
    /**
     * The Bitbucket username when the authType is `BASIC_AUTH`. This parameter is not valid for
     * other types of source providers or connections.
     */
    userName?: pulumi.Input<string>;
}
