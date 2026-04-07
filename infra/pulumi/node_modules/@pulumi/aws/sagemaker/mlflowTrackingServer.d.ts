import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a SageMaker AI MLFlow Tracking Server resource.
 *
 * ## Example Usage
 *
 * ### Cognito Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.MlflowTrackingServer("example", {
 *     trackingServerName: "example",
 *     roleArn: exampleAwsIamRole.arn,
 *     artifactStoreUri: `s3://${exampleAwsS3Bucket.bucket}/path`,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI MLFlow Tracking Servers using the `workteam_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/mlflowTrackingServer:MlflowTrackingServer example example
 * ```
 */
export declare class MlflowTrackingServer extends pulumi.CustomResource {
    /**
     * Get an existing MlflowTrackingServer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MlflowTrackingServerState, opts?: pulumi.CustomResourceOptions): MlflowTrackingServer;
    /**
     * Returns true if the given object is an instance of MlflowTrackingServer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MlflowTrackingServer;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this MLFlow Tracking Server.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The S3 URI for a general purpose bucket to use as the MLflow Tracking Server artifact store.
     */
    readonly artifactStoreUri: pulumi.Output<string>;
    /**
     * A list of Member Definitions that contains objects that identify the workers that make up the work team.
     */
    readonly automaticModelRegistration: pulumi.Output<boolean | undefined>;
    /**
     * The version of MLflow that the tracking server uses. To see which MLflow versions are available to use, see [How it works](https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow.html#mlflow-create-tracking-server-how-it-works).
     */
    readonly mlflowVersion: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) for an IAM role in your account that the MLflow Tracking Server uses to access the artifact store in Amazon S3. The role should have AmazonS3FullAccess permissions. For more information on IAM permissions for tracking server creation, see [Set up IAM permissions for MLflow](https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server-iam.html).
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * A unique string identifying the tracking server name. This string is part of the tracking server ARN.
     */
    readonly trackingServerName: pulumi.Output<string>;
    /**
     * The size of the tracking server you want to create. You can choose between "Small", "Medium", and "Large". The default MLflow Tracking Server configuration size is "Small". You can choose a size depending on the projected use of the tracking server such as the volume of data logged, number of users, and frequency of use.
     */
    readonly trackingServerSize: pulumi.Output<string | undefined>;
    /**
     * The URL to connect to the MLflow user interface for the described tracking server.
     */
    readonly trackingServerUrl: pulumi.Output<string>;
    /**
     * The day and time of the week in Coordinated Universal Time (UTC) 24-hour standard time that weekly maintenance updates are scheduled. For example: TUE:03:30.
     */
    readonly weeklyMaintenanceWindowStart: pulumi.Output<string>;
    /**
     * Create a MlflowTrackingServer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MlflowTrackingServerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MlflowTrackingServer resources.
 */
export interface MlflowTrackingServerState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this MLFlow Tracking Server.
     */
    arn?: pulumi.Input<string>;
    /**
     * The S3 URI for a general purpose bucket to use as the MLflow Tracking Server artifact store.
     */
    artifactStoreUri?: pulumi.Input<string>;
    /**
     * A list of Member Definitions that contains objects that identify the workers that make up the work team.
     */
    automaticModelRegistration?: pulumi.Input<boolean>;
    /**
     * The version of MLflow that the tracking server uses. To see which MLflow versions are available to use, see [How it works](https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow.html#mlflow-create-tracking-server-how-it-works).
     */
    mlflowVersion?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for an IAM role in your account that the MLflow Tracking Server uses to access the artifact store in Amazon S3. The role should have AmazonS3FullAccess permissions. For more information on IAM permissions for tracking server creation, see [Set up IAM permissions for MLflow](https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server-iam.html).
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A unique string identifying the tracking server name. This string is part of the tracking server ARN.
     */
    trackingServerName?: pulumi.Input<string>;
    /**
     * The size of the tracking server you want to create. You can choose between "Small", "Medium", and "Large". The default MLflow Tracking Server configuration size is "Small". You can choose a size depending on the projected use of the tracking server such as the volume of data logged, number of users, and frequency of use.
     */
    trackingServerSize?: pulumi.Input<string>;
    /**
     * The URL to connect to the MLflow user interface for the described tracking server.
     */
    trackingServerUrl?: pulumi.Input<string>;
    /**
     * The day and time of the week in Coordinated Universal Time (UTC) 24-hour standard time that weekly maintenance updates are scheduled. For example: TUE:03:30.
     */
    weeklyMaintenanceWindowStart?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MlflowTrackingServer resource.
 */
export interface MlflowTrackingServerArgs {
    /**
     * The S3 URI for a general purpose bucket to use as the MLflow Tracking Server artifact store.
     */
    artifactStoreUri: pulumi.Input<string>;
    /**
     * A list of Member Definitions that contains objects that identify the workers that make up the work team.
     */
    automaticModelRegistration?: pulumi.Input<boolean>;
    /**
     * The version of MLflow that the tracking server uses. To see which MLflow versions are available to use, see [How it works](https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow.html#mlflow-create-tracking-server-how-it-works).
     */
    mlflowVersion?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for an IAM role in your account that the MLflow Tracking Server uses to access the artifact store in Amazon S3. The role should have AmazonS3FullAccess permissions. For more information on IAM permissions for tracking server creation, see [Set up IAM permissions for MLflow](https://docs.aws.amazon.com/sagemaker/latest/dg/mlflow-create-tracking-server-iam.html).
     */
    roleArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A unique string identifying the tracking server name. This string is part of the tracking server ARN.
     */
    trackingServerName: pulumi.Input<string>;
    /**
     * The size of the tracking server you want to create. You can choose between "Small", "Medium", and "Large". The default MLflow Tracking Server configuration size is "Small". You can choose a size depending on the projected use of the tracking server such as the volume of data logged, number of users, and frequency of use.
     */
    trackingServerSize?: pulumi.Input<string>;
    /**
     * The day and time of the week in Coordinated Universal Time (UTC) 24-hour standard time that weekly maintenance updates are scheduled. For example: TUE:03:30.
     */
    weeklyMaintenanceWindowStart?: pulumi.Input<string>;
}
