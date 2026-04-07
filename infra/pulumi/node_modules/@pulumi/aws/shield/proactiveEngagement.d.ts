import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a AWS Shield Proactive Engagement.
 * Proactive engagement authorizes the Shield Response Team (SRT) to use email and phone to notify contacts about escalations to the SRT and to initiate proactive customer support.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example-role",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Sid: "",
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: "drt.shield.amazonaws.com",
 *             },
 *             Action: "sts:AssumeRole",
 *         }],
 *     }),
 * });
 * const exampleDrtAccessRoleArnAssociation = new aws.shield.DrtAccessRoleArnAssociation("example", {roleArn: exampleRole.arn});
 * const example = new aws.shield.ProactiveEngagement("example", {
 *     enabled: true,
 *     emergencyContacts: [
 *         {
 *             contactNotes: "Notes",
 *             emailAddress: "contact1@example.com",
 *             phoneNumber: "+12358132134",
 *         },
 *         {
 *             contactNotes: "Notes 2",
 *             emailAddress: "contact2@example.com",
 *             phoneNumber: "+12358132134",
 *         },
 *     ],
 * }, {
 *     dependsOn: [exampleDrtAccessRoleArnAssociation],
 * });
 * const exampleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("example", {
 *     role: exampleRole.name,
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AWSShieldDRTAccessPolicy",
 * });
 * const exampleProtectionGroup = new aws.shield.ProtectionGroup("example", {
 *     protectionGroupId: "example",
 *     aggregation: "MAX",
 *     pattern: "ALL",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Shield proactive engagement using the AWS account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:shield/proactiveEngagement:ProactiveEngagement example 123456789012
 * ```
 */
export declare class ProactiveEngagement extends pulumi.CustomResource {
    /**
     * Get an existing ProactiveEngagement resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProactiveEngagementState, opts?: pulumi.CustomResourceOptions): ProactiveEngagement;
    /**
     * Returns true if the given object is an instance of ProactiveEngagement.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProactiveEngagement;
    /**
     * One or more emergency contacts. You must provide at least one phone number in the emergency contact list. See `emergencyContacts`.
     */
    readonly emergencyContacts: pulumi.Output<outputs.shield.ProactiveEngagementEmergencyContact[] | undefined>;
    /**
     * Boolean value indicating if Proactive Engagement should be enabled or not.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Create a ProactiveEngagement resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProactiveEngagementArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProactiveEngagement resources.
 */
export interface ProactiveEngagementState {
    /**
     * One or more emergency contacts. You must provide at least one phone number in the emergency contact list. See `emergencyContacts`.
     */
    emergencyContacts?: pulumi.Input<pulumi.Input<inputs.shield.ProactiveEngagementEmergencyContact>[]>;
    /**
     * Boolean value indicating if Proactive Engagement should be enabled or not.
     */
    enabled?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a ProactiveEngagement resource.
 */
export interface ProactiveEngagementArgs {
    /**
     * One or more emergency contacts. You must provide at least one phone number in the emergency contact list. See `emergencyContacts`.
     */
    emergencyContacts?: pulumi.Input<pulumi.Input<inputs.shield.ProactiveEngagementEmergencyContact>[]>;
    /**
     * Boolean value indicating if Proactive Engagement should be enabled or not.
     */
    enabled: pulumi.Input<boolean>;
}
