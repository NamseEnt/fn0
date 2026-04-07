import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amplify Branch resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amplify.App("example", {name: "app"});
 * const master = new aws.amplify.Branch("master", {
 *     appId: example.id,
 *     branchName: "master",
 *     framework: "React",
 *     stage: "PRODUCTION",
 *     environmentVariables: {
 *         REACT_APP_API_SERVER: "https://api.example.com",
 *     },
 * });
 * ```
 *
 * ### Basic Authentication
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.amplify.App("example", {name: "app"});
 * const master = new aws.amplify.Branch("master", {
 *     appId: example.id,
 *     branchName: "master",
 *     enableBasicAuth: true,
 *     basicAuthCredentials: std.base64encode({
 *         input: "username:password",
 *     }).then(invoke => invoke.result),
 * });
 * ```
 *
 * ### Notifications
 *
 * Amplify Console uses EventBridge (formerly known as CloudWatch Events) and SNS for email notifications.  To implement the same functionality, you need to set `enableNotification` in a `aws.amplify.Branch` resource, as well as creating an EventBridge Rule, an SNS topic, and SNS subscriptions.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amplify.App("example", {name: "app"});
 * const master = new aws.amplify.Branch("master", {
 *     appId: example.id,
 *     branchName: "master",
 *     enableNotification: true,
 * });
 * // EventBridge Rule for Amplify notifications
 * const amplifyAppMasterEventRule = new aws.cloudwatch.EventRule("amplify_app_master", {
 *     name: pulumi.interpolate`amplify-${app.id}-${master.branchName}-branch-notification`,
 *     description: pulumi.interpolate`AWS Amplify build notifications for :  App: ${app.id} Branch: ${master.branchName}`,
 *     eventPattern: pulumi.jsonStringify({
 *         detail: {
 *             appId: [example.id],
 *             branchName: [master.branchName],
 *             jobStatus: [
 *                 "SUCCEED",
 *                 "FAILED",
 *                 "STARTED",
 *             ],
 *         },
 *         "detail-type": ["Amplify Deployment Status Change"],
 *         source: ["aws.amplify"],
 *     }),
 * });
 * // SNS Topic for Amplify notifications
 * const amplifyAppMasterTopic = new aws.sns.Topic("amplify_app_master", {name: pulumi.interpolate`amplify-${app.id}_${master.branchName}`});
 * const amplifyAppMasterEventTarget = new aws.cloudwatch.EventTarget("amplify_app_master", {
 *     rule: amplifyAppMasterEventRule.name,
 *     targetId: master.branchName,
 *     arn: amplifyAppMasterTopic.arn,
 *     inputTransformer: {
 *         inputPaths: {
 *             jobId: "$.detail.jobId",
 *             appId: "$.detail.appId",
 *             region: "$.region",
 *             branch: "$.detail.branchName",
 *             status: "$.detail.jobStatus",
 *         },
 *         inputTemplate: "\"Build notification from the AWS Amplify Console for app: https://<branch>.<appId>.amplifyapp.com/. Your build status is <status>. Go to https://console.aws.amazon.com/amplify/home?region=<region>#<appId>/<branch>/<jobId> to view details on your build. \"",
 *     },
 * });
 * const amplifyAppMaster = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         sid: pulumi.interpolate`Allow_Publish_Events ${master.arn}`,
 *         effect: "Allow",
 *         actions: ["SNS:Publish"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["events.amazonaws.com"],
 *         }],
 *         resources: [amplifyAppMasterTopic.arn],
 *     }],
 * });
 * const amplifyAppMasterTopicPolicy = new aws.sns.TopicPolicy("amplify_app_master", {
 *     arn: amplifyAppMasterTopic.arn,
 *     policy: amplifyAppMaster.apply(amplifyAppMaster => amplifyAppMaster.json),
 * });
 * const _this = new aws.sns.TopicSubscription("this", {
 *     topic: amplifyAppMasterTopic.arn,
 *     protocol: "email",
 *     endpoint: "user@acme.com",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amplify branch using `app_id` and `branch_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:amplify/branch:Branch master d2ypk4k47z8u6/master
 * ```
 */
export declare class Branch extends pulumi.CustomResource {
    /**
     * Get an existing Branch resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BranchState, opts?: pulumi.CustomResourceOptions): Branch;
    /**
     * Returns true if the given object is an instance of Branch.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Branch;
    /**
     * Unique ID for an Amplify app.
     */
    readonly appId: pulumi.Output<string>;
    /**
     * ARN for the branch.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A list of custom resources that are linked to this branch.
     */
    readonly associatedResources: pulumi.Output<string[]>;
    /**
     * ARN for a backend environment that is part of an Amplify app.
     */
    readonly backendEnvironmentArn: pulumi.Output<string | undefined>;
    /**
     * Basic authorization credentials for the branch.
     */
    readonly basicAuthCredentials: pulumi.Output<string | undefined>;
    /**
     * Name for the branch.
     */
    readonly branchName: pulumi.Output<string>;
    /**
     * Custom domains for the branch.
     */
    readonly customDomains: pulumi.Output<string[]>;
    /**
     * Description for the branch.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Destination branch if the branch is a pull request branch.
     */
    readonly destinationBranch: pulumi.Output<string>;
    /**
     * Display name for a branch. This is used as the default domain prefix.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * Enables auto building for the branch.
     */
    readonly enableAutoBuild: pulumi.Output<boolean | undefined>;
    /**
     * Enables basic authorization for the branch.
     */
    readonly enableBasicAuth: pulumi.Output<boolean | undefined>;
    /**
     * Enables notifications for the branch.
     */
    readonly enableNotification: pulumi.Output<boolean | undefined>;
    /**
     * Enables performance mode for the branch.
     */
    readonly enablePerformanceMode: pulumi.Output<boolean | undefined>;
    /**
     * Enables pull request previews for this branch.
     */
    readonly enablePullRequestPreview: pulumi.Output<boolean | undefined>;
    /**
     * Enables skew protection for the branch.
     */
    readonly enableSkewProtection: pulumi.Output<boolean | undefined>;
    /**
     * Environment variables for the branch.
     */
    readonly environmentVariables: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Framework for the branch.
     */
    readonly framework: pulumi.Output<string | undefined>;
    /**
     * Amplify environment name for the pull request.
     */
    readonly pullRequestEnvironmentName: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Source branch if the branch is a pull request branch.
     */
    readonly sourceBranch: pulumi.Output<string>;
    /**
     * Describes the current stage for the branch. Valid values: `PRODUCTION`, `BETA`, `DEVELOPMENT`, `EXPERIMENTAL`, `PULL_REQUEST`.
     */
    readonly stage: pulumi.Output<string | undefined>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Content Time To Live (TTL) for the website in seconds.
     */
    readonly ttl: pulumi.Output<string | undefined>;
    /**
     * Create a Branch resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BranchArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Branch resources.
 */
export interface BranchState {
    /**
     * Unique ID for an Amplify app.
     */
    appId?: pulumi.Input<string>;
    /**
     * ARN for the branch.
     */
    arn?: pulumi.Input<string>;
    /**
     * A list of custom resources that are linked to this branch.
     */
    associatedResources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN for a backend environment that is part of an Amplify app.
     */
    backendEnvironmentArn?: pulumi.Input<string>;
    /**
     * Basic authorization credentials for the branch.
     */
    basicAuthCredentials?: pulumi.Input<string>;
    /**
     * Name for the branch.
     */
    branchName?: pulumi.Input<string>;
    /**
     * Custom domains for the branch.
     */
    customDomains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Description for the branch.
     */
    description?: pulumi.Input<string>;
    /**
     * Destination branch if the branch is a pull request branch.
     */
    destinationBranch?: pulumi.Input<string>;
    /**
     * Display name for a branch. This is used as the default domain prefix.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Enables auto building for the branch.
     */
    enableAutoBuild?: pulumi.Input<boolean>;
    /**
     * Enables basic authorization for the branch.
     */
    enableBasicAuth?: pulumi.Input<boolean>;
    /**
     * Enables notifications for the branch.
     */
    enableNotification?: pulumi.Input<boolean>;
    /**
     * Enables performance mode for the branch.
     */
    enablePerformanceMode?: pulumi.Input<boolean>;
    /**
     * Enables pull request previews for this branch.
     */
    enablePullRequestPreview?: pulumi.Input<boolean>;
    /**
     * Enables skew protection for the branch.
     */
    enableSkewProtection?: pulumi.Input<boolean>;
    /**
     * Environment variables for the branch.
     */
    environmentVariables?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Framework for the branch.
     */
    framework?: pulumi.Input<string>;
    /**
     * Amplify environment name for the pull request.
     */
    pullRequestEnvironmentName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Source branch if the branch is a pull request branch.
     */
    sourceBranch?: pulumi.Input<string>;
    /**
     * Describes the current stage for the branch. Valid values: `PRODUCTION`, `BETA`, `DEVELOPMENT`, `EXPERIMENTAL`, `PULL_REQUEST`.
     */
    stage?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Content Time To Live (TTL) for the website in seconds.
     */
    ttl?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Branch resource.
 */
export interface BranchArgs {
    /**
     * Unique ID for an Amplify app.
     */
    appId: pulumi.Input<string>;
    /**
     * ARN for a backend environment that is part of an Amplify app.
     */
    backendEnvironmentArn?: pulumi.Input<string>;
    /**
     * Basic authorization credentials for the branch.
     */
    basicAuthCredentials?: pulumi.Input<string>;
    /**
     * Name for the branch.
     */
    branchName: pulumi.Input<string>;
    /**
     * Description for the branch.
     */
    description?: pulumi.Input<string>;
    /**
     * Display name for a branch. This is used as the default domain prefix.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Enables auto building for the branch.
     */
    enableAutoBuild?: pulumi.Input<boolean>;
    /**
     * Enables basic authorization for the branch.
     */
    enableBasicAuth?: pulumi.Input<boolean>;
    /**
     * Enables notifications for the branch.
     */
    enableNotification?: pulumi.Input<boolean>;
    /**
     * Enables performance mode for the branch.
     */
    enablePerformanceMode?: pulumi.Input<boolean>;
    /**
     * Enables pull request previews for this branch.
     */
    enablePullRequestPreview?: pulumi.Input<boolean>;
    /**
     * Enables skew protection for the branch.
     */
    enableSkewProtection?: pulumi.Input<boolean>;
    /**
     * Environment variables for the branch.
     */
    environmentVariables?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Framework for the branch.
     */
    framework?: pulumi.Input<string>;
    /**
     * Amplify environment name for the pull request.
     */
    pullRequestEnvironmentName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Describes the current stage for the branch. Valid values: `PRODUCTION`, `BETA`, `DEVELOPMENT`, `EXPERIMENTAL`, `PULL_REQUEST`.
     */
    stage?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Content Time To Live (TTL) for the website in seconds.
     */
    ttl?: pulumi.Input<string>;
}
