import * as pulumi from "@pulumi/pulumi";
import { Bucket } from "../s3";
import { Application } from "./index";
/**
 * Provides an Elastic Beanstalk Application Version Resource. Elastic Beanstalk allows
 * you to deploy and manage applications in the AWS cloud without worrying about
 * the infrastructure that runs those applications.
 *
 * This resource creates a Beanstalk Application Version that can be deployed to a Beanstalk
 * Environment.
 *
 * > **NOTE on Application Version Resource:**  When using the Application Version resource with multiple
 * Elastic Beanstalk Environments it is possible that an error may be returned
 * when attempting to delete an Application Version while it is still in use by a different environment.
 * To work around this you can either create each environment in a separate AWS account or create your `aws.elasticbeanstalk.ApplicationVersion` resources with a unique names in your Elastic Beanstalk Application. For example &lt;revision&gt;-&lt;environment&gt;.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.s3.Bucket("default", {bucket: "tftest.applicationversion.bucket"});
 * const defaultBucketObjectv2 = new aws.s3.BucketObjectv2("default", {
 *     bucket: _default.id,
 *     key: "beanstalk/go-v1.zip",
 *     source: new pulumi.asset.FileAsset("go-v1.zip"),
 * });
 * const defaultApplication = new aws.elasticbeanstalk.Application("default", {
 *     name: "tf-test-name",
 *     description: "tf-test-desc",
 * });
 * const defaultApplicationVersion = new aws.elasticbeanstalk.ApplicationVersion("default", {
 *     name: "tf-test-version-label",
 *     application: "tf-test-name",
 *     description: "application version",
 *     bucket: _default.id,
 *     key: defaultBucketObjectv2.key,
 * });
 * ```
 */
export declare class ApplicationVersion extends pulumi.CustomResource {
    /**
     * Get an existing ApplicationVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationVersionState, opts?: pulumi.CustomResourceOptions): ApplicationVersion;
    /**
     * Returns true if the given object is an instance of ApplicationVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApplicationVersion;
    /**
     * Name of the Beanstalk Application the version is associated with.
     */
    readonly application: pulumi.Output<string>;
    /**
     * ARN assigned by AWS for this Elastic Beanstalk Application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * S3 bucket that contains the Application Version source bundle.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Short description of the Application Version.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * On delete, force an Application Version to be deleted when it may be in use by multiple Elastic Beanstalk Environments.
     */
    readonly forceDelete: pulumi.Output<boolean | undefined>;
    /**
     * S3 object that is the Application Version source bundle.
     */
    readonly key: pulumi.Output<string>;
    /**
     * Unique name for the this Application Version.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Pre-processes and validates the environment manifest (env.yaml ) and configuration files (*.config files in the .ebextensions folder) in the source bundle. Validating configuration files can identify issues prior to deploying the application version to an environment. You must turn processing on for application versions that you create using AWS CodeBuild or AWS CodeCommit. For application versions built from a source bundle in Amazon S3, processing is optional. It validates Elastic Beanstalk configuration files. It doesn’t validate your application’s configuration files, like proxy server or Docker configuration.
     */
    readonly process: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of tags for the Elastic Beanstalk Application Version. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ApplicationVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApplicationVersion resources.
 */
export interface ApplicationVersionState {
    /**
     * Name of the Beanstalk Application the version is associated with.
     */
    application?: pulumi.Input<string | Application>;
    /**
     * ARN assigned by AWS for this Elastic Beanstalk Application.
     */
    arn?: pulumi.Input<string>;
    /**
     * S3 bucket that contains the Application Version source bundle.
     */
    bucket?: pulumi.Input<string | Bucket>;
    /**
     * Short description of the Application Version.
     */
    description?: pulumi.Input<string>;
    /**
     * On delete, force an Application Version to be deleted when it may be in use by multiple Elastic Beanstalk Environments.
     */
    forceDelete?: pulumi.Input<boolean>;
    /**
     * S3 object that is the Application Version source bundle.
     */
    key?: pulumi.Input<string>;
    /**
     * Unique name for the this Application Version.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Pre-processes and validates the environment manifest (env.yaml ) and configuration files (*.config files in the .ebextensions folder) in the source bundle. Validating configuration files can identify issues prior to deploying the application version to an environment. You must turn processing on for application versions that you create using AWS CodeBuild or AWS CodeCommit. For application versions built from a source bundle in Amazon S3, processing is optional. It validates Elastic Beanstalk configuration files. It doesn’t validate your application’s configuration files, like proxy server or Docker configuration.
     */
    process?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of tags for the Elastic Beanstalk Application Version. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ApplicationVersion resource.
 */
export interface ApplicationVersionArgs {
    /**
     * Name of the Beanstalk Application the version is associated with.
     */
    application: pulumi.Input<string | Application>;
    /**
     * S3 bucket that contains the Application Version source bundle.
     */
    bucket: pulumi.Input<string | Bucket>;
    /**
     * Short description of the Application Version.
     */
    description?: pulumi.Input<string>;
    /**
     * On delete, force an Application Version to be deleted when it may be in use by multiple Elastic Beanstalk Environments.
     */
    forceDelete?: pulumi.Input<boolean>;
    /**
     * S3 object that is the Application Version source bundle.
     */
    key: pulumi.Input<string>;
    /**
     * Unique name for the this Application Version.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Pre-processes and validates the environment manifest (env.yaml ) and configuration files (*.config files in the .ebextensions folder) in the source bundle. Validating configuration files can identify issues prior to deploying the application version to an environment. You must turn processing on for application versions that you create using AWS CodeBuild or AWS CodeCommit. For application versions built from a source bundle in Amazon S3, processing is optional. It validates Elastic Beanstalk configuration files. It doesn’t validate your application’s configuration files, like proxy server or Docker configuration.
     */
    process?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of tags for the Elastic Beanstalk Application Version. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
