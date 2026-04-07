import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Workteam resource.
 *
 * ## Example Usage
 *
 * ### Cognito Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Workteam("example", {
 *     workteamName: "example",
 *     workforceName: exampleAwsSagemakerWorkforce.id,
 *     description: "example",
 *     memberDefinitions: [{
 *         cognitoMemberDefinition: {
 *             clientId: exampleAwsCognitoUserPoolClient.id,
 *             userPool: exampleAwsCognitoUserPoolDomain.userPoolId,
 *             userGroup: exampleAwsCognitoUserGroup.name,
 *         },
 *     }],
 * });
 * ```
 *
 * ### Oidc Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Workteam("example", {
 *     workteamName: "example",
 *     workforceName: exampleAwsSagemakerWorkforce.id,
 *     description: "example",
 *     memberDefinitions: [{
 *         oidcMemberDefinition: {
 *             groups: ["example"],
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Workteams using the `workteam_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/workteam:Workteam example example
 * ```
 */
export declare class Workteam extends pulumi.CustomResource {
    /**
     * Get an existing Workteam resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkteamState, opts?: pulumi.CustomResourceOptions): Workteam;
    /**
     * Returns true if the given object is an instance of Workteam.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Workteam;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Workteam.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the work team.
     */
    readonly description: pulumi.Output<string>;
    /**
     * A list of Member Definitions that contains objects that identify the workers that make up the work team. Workforces can be created using Amazon Cognito or your own OIDC Identity Provider (IdP). For private workforces created using Amazon Cognito use `cognitoMemberDefinition`. For workforces created using your own OIDC identity provider (IdP) use `oidcMemberDefinition`. Do not provide input for both of these parameters in a single request. see Member Definition details below.
     */
    readonly memberDefinitions: pulumi.Output<outputs.sagemaker.WorkteamMemberDefinition[]>;
    /**
     * Configures notification of workers regarding available or expiring work items. see Notification Configuration details below.
     */
    readonly notificationConfiguration: pulumi.Output<outputs.sagemaker.WorkteamNotificationConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The subdomain for your OIDC Identity Provider.
     */
    readonly subdomain: pulumi.Output<string>;
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
     * Use this optional parameter to constrain access to an Amazon S3 resource based on the IP address using supported IAM global condition keys. The Amazon S3 resource is accessed in the worker portal using a Amazon S3 presigned URL. see Worker Access Configuration details below.
     */
    readonly workerAccessConfiguration: pulumi.Output<outputs.sagemaker.WorkteamWorkerAccessConfiguration>;
    /**
     * The name of the workforce.
     */
    readonly workforceName: pulumi.Output<string | undefined>;
    /**
     * The name of the Workteam (must be unique).
     */
    readonly workteamName: pulumi.Output<string>;
    /**
     * Create a Workteam resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkteamArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Workteam resources.
 */
export interface WorkteamState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Workteam.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the work team.
     */
    description?: pulumi.Input<string>;
    /**
     * A list of Member Definitions that contains objects that identify the workers that make up the work team. Workforces can be created using Amazon Cognito or your own OIDC Identity Provider (IdP). For private workforces created using Amazon Cognito use `cognitoMemberDefinition`. For workforces created using your own OIDC identity provider (IdP) use `oidcMemberDefinition`. Do not provide input for both of these parameters in a single request. see Member Definition details below.
     */
    memberDefinitions?: pulumi.Input<pulumi.Input<inputs.sagemaker.WorkteamMemberDefinition>[]>;
    /**
     * Configures notification of workers regarding available or expiring work items. see Notification Configuration details below.
     */
    notificationConfiguration?: pulumi.Input<inputs.sagemaker.WorkteamNotificationConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The subdomain for your OIDC Identity Provider.
     */
    subdomain?: pulumi.Input<string>;
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
     * Use this optional parameter to constrain access to an Amazon S3 resource based on the IP address using supported IAM global condition keys. The Amazon S3 resource is accessed in the worker portal using a Amazon S3 presigned URL. see Worker Access Configuration details below.
     */
    workerAccessConfiguration?: pulumi.Input<inputs.sagemaker.WorkteamWorkerAccessConfiguration>;
    /**
     * The name of the workforce.
     */
    workforceName?: pulumi.Input<string>;
    /**
     * The name of the Workteam (must be unique).
     */
    workteamName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Workteam resource.
 */
export interface WorkteamArgs {
    /**
     * A description of the work team.
     */
    description: pulumi.Input<string>;
    /**
     * A list of Member Definitions that contains objects that identify the workers that make up the work team. Workforces can be created using Amazon Cognito or your own OIDC Identity Provider (IdP). For private workforces created using Amazon Cognito use `cognitoMemberDefinition`. For workforces created using your own OIDC identity provider (IdP) use `oidcMemberDefinition`. Do not provide input for both of these parameters in a single request. see Member Definition details below.
     */
    memberDefinitions: pulumi.Input<pulumi.Input<inputs.sagemaker.WorkteamMemberDefinition>[]>;
    /**
     * Configures notification of workers regarding available or expiring work items. see Notification Configuration details below.
     */
    notificationConfiguration?: pulumi.Input<inputs.sagemaker.WorkteamNotificationConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Use this optional parameter to constrain access to an Amazon S3 resource based on the IP address using supported IAM global condition keys. The Amazon S3 resource is accessed in the worker portal using a Amazon S3 presigned URL. see Worker Access Configuration details below.
     */
    workerAccessConfiguration?: pulumi.Input<inputs.sagemaker.WorkteamWorkerAccessConfiguration>;
    /**
     * The name of the workforce.
     */
    workforceName?: pulumi.Input<string>;
    /**
     * The name of the Workteam (must be unique).
     */
    workteamName: pulumi.Input<string>;
}
