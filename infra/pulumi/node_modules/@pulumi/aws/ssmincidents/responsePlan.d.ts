import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage response plans in AWS Systems Manager Incident Manager.
 *
 * > NOTE: A response plan implicitly depends on a replication set. If you configured your replication set in Pulumi, we recommend you add it to the `dependsOn` argument for the ResponsePlan Resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssmincidents.ResponsePlan("example", {
 *     name: "name",
 *     incidentTemplate: {
 *         title: "title",
 *         impact: 3,
 *     },
 *     tags: {
 *         key: "value",
 *     },
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ### Usage With All Fields
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssmincidents.ResponsePlan("example", {
 *     name: "name",
 *     incidentTemplate: {
 *         title: "title",
 *         impact: 3,
 *         dedupeString: "dedupe",
 *         incidentTags: {
 *             key: "value",
 *         },
 *         notificationTargets: [
 *             {
 *                 snsTopicArn: example1.arn,
 *             },
 *             {
 *                 snsTopicArn: example2.arn,
 *             },
 *         ],
 *         summary: "summary",
 *     },
 *     displayName: "display name",
 *     chatChannels: [topic.arn],
 *     engagements: ["arn:aws:ssm-contacts:us-east-2:111122223333:contact/test1"],
 *     action: {
 *         ssmAutomations: [{
 *             documentName: document1.name,
 *             roleArn: role1.arn,
 *             documentVersion: "version1",
 *             targetAccount: "RESPONSE_PLAN_OWNER_ACCOUNT",
 *             parameters: [
 *                 {
 *                     name: "key",
 *                     values: [
 *                         "value1",
 *                         "value2",
 *                     ],
 *                 },
 *                 {
 *                     name: "foo",
 *                     values: ["bar"],
 *                 },
 *             ],
 *             dynamicParameters: {
 *                 someKey: "INVOLVED_RESOURCES",
 *                 anotherKey: "INCIDENT_RECORD_ARN",
 *             },
 *         }],
 *     },
 *     integration: {
 *         pagerduties: [{
 *             name: "pagerdutyIntergration",
 *             serviceId: "example",
 *             secretId: "example",
 *         }],
 *     },
 *     tags: {
 *         key: "value",
 *     },
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an Incident Manager response plan using the response plan ARN. You can find the response plan ARN in the AWS Management Console. For example:
 *
 * ```sh
 * $ pulumi import aws:ssmincidents/responsePlan:ResponsePlan responsePlanName ARNValue
 * ```
 */
export declare class ResponsePlan extends pulumi.CustomResource {
    /**
     * Get an existing ResponsePlan resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResponsePlanState, opts?: pulumi.CustomResourceOptions): ResponsePlan;
    /**
     * Returns true if the given object is an instance of ResponsePlan.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResponsePlan;
    /**
     * The actions that the response plan starts at the beginning of an incident.
     */
    readonly action: pulumi.Output<outputs.ssmincidents.ResponsePlanAction | undefined>;
    /**
     * The ARN of the response plan.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Chatbot chat channel used for collaboration during an incident.
     */
    readonly chatChannels: pulumi.Output<string[] | undefined>;
    /**
     * The long format of the response plan name. This field can contain spaces.
     */
    readonly displayName: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) for the contacts and escalation plans that the response plan engages during an incident.
     */
    readonly engagements: pulumi.Output<string[] | undefined>;
    /**
     * The `incidentTemplate` configuration block is required and supports the following arguments:
     */
    readonly incidentTemplate: pulumi.Output<outputs.ssmincidents.ResponsePlanIncidentTemplate>;
    /**
     * Information about third-party services integrated into the response plan. The following values are supported:
     */
    readonly integration: pulumi.Output<outputs.ssmincidents.ResponsePlanIntegration | undefined>;
    /**
     * The name of the response plan.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The tags applied to the response plan.
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
     * Create a ResponsePlan resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResponsePlanArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResponsePlan resources.
 */
export interface ResponsePlanState {
    /**
     * The actions that the response plan starts at the beginning of an incident.
     */
    action?: pulumi.Input<inputs.ssmincidents.ResponsePlanAction>;
    /**
     * The ARN of the response plan.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Chatbot chat channel used for collaboration during an incident.
     */
    chatChannels?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The long format of the response plan name. This field can contain spaces.
     */
    displayName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the contacts and escalation plans that the response plan engages during an incident.
     */
    engagements?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The `incidentTemplate` configuration block is required and supports the following arguments:
     */
    incidentTemplate?: pulumi.Input<inputs.ssmincidents.ResponsePlanIncidentTemplate>;
    /**
     * Information about third-party services integrated into the response plan. The following values are supported:
     */
    integration?: pulumi.Input<inputs.ssmincidents.ResponsePlanIntegration>;
    /**
     * The name of the response plan.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The tags applied to the response plan.
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
}
/**
 * The set of arguments for constructing a ResponsePlan resource.
 */
export interface ResponsePlanArgs {
    /**
     * The actions that the response plan starts at the beginning of an incident.
     */
    action?: pulumi.Input<inputs.ssmincidents.ResponsePlanAction>;
    /**
     * The Chatbot chat channel used for collaboration during an incident.
     */
    chatChannels?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The long format of the response plan name. This field can contain spaces.
     */
    displayName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the contacts and escalation plans that the response plan engages during an incident.
     */
    engagements?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The `incidentTemplate` configuration block is required and supports the following arguments:
     */
    incidentTemplate: pulumi.Input<inputs.ssmincidents.ResponsePlanIncidentTemplate>;
    /**
     * Information about third-party services integrated into the response plan. The following values are supported:
     */
    integration?: pulumi.Input<inputs.ssmincidents.ResponsePlanIntegration>;
    /**
     * The name of the response plan.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The tags applied to the response plan.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
