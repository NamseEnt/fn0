import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an SNS platform application resource
 *
 * ## Example Usage
 *
 * ### Apple Push Notification Service (APNS) using certificate-based authentication
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const apnsApplication = new aws.sns.PlatformApplication("apns_application", {
 *     name: "apns_application",
 *     platform: "APNS",
 *     platformCredential: "<APNS PRIVATE KEY>",
 *     platformPrincipal: "<APNS CERTIFICATE>",
 * });
 * ```
 *
 * ### Apple Push Notification Service (APNS) using token-based authentication
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const apnsApplication = new aws.sns.PlatformApplication("apns_application", {
 *     name: "apns_application",
 *     platform: "APNS",
 *     platformCredential: "<APNS SIGNING KEY>",
 *     platformPrincipal: "<APNS SIGNING KEY ID>",
 *     applePlatformTeamId: "<APPLE TEAM ID>",
 *     applePlatformBundleId: "<APPLE BUNDLE ID>",
 * });
 * ```
 *
 * ### Google Cloud Messaging (GCM)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const gcmApplication = new aws.sns.PlatformApplication("gcm_application", {
 *     name: "gcm_application",
 *     platform: "GCM",
 *     platformCredential: "<GCM API KEY>",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SNS platform applications using the ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:sns/platformApplication:PlatformApplication gcm_application arn:aws:sns:us-west-2:123456789012:app/GCM/gcm_application
 * ```
 */
export declare class PlatformApplication extends pulumi.CustomResource {
    /**
     * Get an existing PlatformApplication resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PlatformApplicationState, opts?: pulumi.CustomResourceOptions): PlatformApplication;
    /**
     * Returns true if the given object is an instance of PlatformApplication.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PlatformApplication;
    /**
     * The bundle identifier that's assigned to your iOS app. May only include alphanumeric characters, hyphens (-), and periods (.).
     */
    readonly applePlatformBundleId: pulumi.Output<string | undefined>;
    /**
     * The identifier that's assigned to your Apple developer account team. Must be 10 alphanumeric characters.
     */
    readonly applePlatformTeamId: pulumi.Output<string | undefined>;
    /**
     * The ARN of the SNS platform application
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the SNS Topic triggered when a delivery to any of the platform endpoints associated with your platform application encounters a permanent failure.
     */
    readonly eventDeliveryFailureTopicArn: pulumi.Output<string | undefined>;
    /**
     * The ARN of the SNS Topic triggered when a new platform endpoint is added to your platform application.
     */
    readonly eventEndpointCreatedTopicArn: pulumi.Output<string | undefined>;
    /**
     * The ARN of the SNS Topic triggered when an existing platform endpoint is deleted from your platform application.
     */
    readonly eventEndpointDeletedTopicArn: pulumi.Output<string | undefined>;
    /**
     * The ARN of the SNS Topic triggered when an existing platform endpoint is changed from your platform application.
     */
    readonly eventEndpointUpdatedTopicArn: pulumi.Output<string | undefined>;
    /**
     * The IAM role ARN permitted to receive failure feedback for this application and give SNS write access to use CloudWatch logs on your behalf.
     */
    readonly failureFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The friendly name for the SNS platform application
     */
    readonly name: pulumi.Output<string>;
    /**
     * The platform that the app is registered with. See [Platform](http://docs.aws.amazon.com/sns/latest/dg/mobile-push-send-register.html) for supported platforms.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Application Platform credential. See [Credential](http://docs.aws.amazon.com/sns/latest/dg/mobile-push-send-register.html) for type of credential required for platform. The value of this attribute when stored into the state is only a hash of the real value, so therefore it is not practical to use this as an attribute for other resources.
     */
    readonly platformCredential: pulumi.Output<string>;
    /**
     * Application Platform principal. See [Principal](http://docs.aws.amazon.com/sns/latest/api/API_CreatePlatformApplication.html) for type of principal required for platform. The value of this attribute when stored into the state is only a hash of the real value, so therefore it is not practical to use this as an attribute for other resources.
     */
    readonly platformPrincipal: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role ARN permitted to receive success feedback for this application and give SNS write access to use CloudWatch logs on your behalf.
     */
    readonly successFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The sample rate percentage (0-100) of successfully delivered messages.
     *
     * The following attributes are needed only when using APNS token credentials:
     */
    readonly successFeedbackSampleRate: pulumi.Output<string | undefined>;
    /**
     * Create a PlatformApplication resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PlatformApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PlatformApplication resources.
 */
export interface PlatformApplicationState {
    /**
     * The bundle identifier that's assigned to your iOS app. May only include alphanumeric characters, hyphens (-), and periods (.).
     */
    applePlatformBundleId?: pulumi.Input<string>;
    /**
     * The identifier that's assigned to your Apple developer account team. Must be 10 alphanumeric characters.
     */
    applePlatformTeamId?: pulumi.Input<string>;
    /**
     * The ARN of the SNS platform application
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when a delivery to any of the platform endpoints associated with your platform application encounters a permanent failure.
     */
    eventDeliveryFailureTopicArn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when a new platform endpoint is added to your platform application.
     */
    eventEndpointCreatedTopicArn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when an existing platform endpoint is deleted from your platform application.
     */
    eventEndpointDeletedTopicArn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when an existing platform endpoint is changed from your platform application.
     */
    eventEndpointUpdatedTopicArn?: pulumi.Input<string>;
    /**
     * The IAM role ARN permitted to receive failure feedback for this application and give SNS write access to use CloudWatch logs on your behalf.
     */
    failureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The friendly name for the SNS platform application
     */
    name?: pulumi.Input<string>;
    /**
     * The platform that the app is registered with. See [Platform](http://docs.aws.amazon.com/sns/latest/dg/mobile-push-send-register.html) for supported platforms.
     */
    platform?: pulumi.Input<string>;
    /**
     * Application Platform credential. See [Credential](http://docs.aws.amazon.com/sns/latest/dg/mobile-push-send-register.html) for type of credential required for platform. The value of this attribute when stored into the state is only a hash of the real value, so therefore it is not practical to use this as an attribute for other resources.
     */
    platformCredential?: pulumi.Input<string>;
    /**
     * Application Platform principal. See [Principal](http://docs.aws.amazon.com/sns/latest/api/API_CreatePlatformApplication.html) for type of principal required for platform. The value of this attribute when stored into the state is only a hash of the real value, so therefore it is not practical to use this as an attribute for other resources.
     */
    platformPrincipal?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role ARN permitted to receive success feedback for this application and give SNS write access to use CloudWatch logs on your behalf.
     */
    successFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The sample rate percentage (0-100) of successfully delivered messages.
     *
     * The following attributes are needed only when using APNS token credentials:
     */
    successFeedbackSampleRate?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PlatformApplication resource.
 */
export interface PlatformApplicationArgs {
    /**
     * The bundle identifier that's assigned to your iOS app. May only include alphanumeric characters, hyphens (-), and periods (.).
     */
    applePlatformBundleId?: pulumi.Input<string>;
    /**
     * The identifier that's assigned to your Apple developer account team. Must be 10 alphanumeric characters.
     */
    applePlatformTeamId?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when a delivery to any of the platform endpoints associated with your platform application encounters a permanent failure.
     */
    eventDeliveryFailureTopicArn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when a new platform endpoint is added to your platform application.
     */
    eventEndpointCreatedTopicArn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when an existing platform endpoint is deleted from your platform application.
     */
    eventEndpointDeletedTopicArn?: pulumi.Input<string>;
    /**
     * The ARN of the SNS Topic triggered when an existing platform endpoint is changed from your platform application.
     */
    eventEndpointUpdatedTopicArn?: pulumi.Input<string>;
    /**
     * The IAM role ARN permitted to receive failure feedback for this application and give SNS write access to use CloudWatch logs on your behalf.
     */
    failureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The friendly name for the SNS platform application
     */
    name?: pulumi.Input<string>;
    /**
     * The platform that the app is registered with. See [Platform](http://docs.aws.amazon.com/sns/latest/dg/mobile-push-send-register.html) for supported platforms.
     */
    platform: pulumi.Input<string>;
    /**
     * Application Platform credential. See [Credential](http://docs.aws.amazon.com/sns/latest/dg/mobile-push-send-register.html) for type of credential required for platform. The value of this attribute when stored into the state is only a hash of the real value, so therefore it is not practical to use this as an attribute for other resources.
     */
    platformCredential: pulumi.Input<string>;
    /**
     * Application Platform principal. See [Principal](http://docs.aws.amazon.com/sns/latest/api/API_CreatePlatformApplication.html) for type of principal required for platform. The value of this attribute when stored into the state is only a hash of the real value, so therefore it is not practical to use this as an attribute for other resources.
     */
    platformPrincipal?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role ARN permitted to receive success feedback for this application and give SNS write access to use CloudWatch logs on your behalf.
     */
    successFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The sample rate percentage (0-100) of successfully delivered messages.
     *
     * The following attributes are needed only when using APNS token credentials:
     */
    successFeedbackSampleRate?: pulumi.Input<string>;
}
