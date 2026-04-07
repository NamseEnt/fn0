import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an IoT fleet provisioning template. For more info, see the AWS documentation on [fleet provisioning](https://docs.aws.amazon.com/iot/latest/developerguide/provision-wo-cert.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const iotAssumeRolePolicy = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["iot.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const iotFleetProvisioning = new aws.iam.Role("iot_fleet_provisioning", {
 *     name: "IoTProvisioningServiceRole",
 *     path: "/service-role/",
 *     assumeRolePolicy: iotAssumeRolePolicy.then(iotAssumeRolePolicy => iotAssumeRolePolicy.json),
 * });
 * const iotFleetProvisioningRegistration = new aws.iam.RolePolicyAttachment("iot_fleet_provisioning_registration", {
 *     role: iotFleetProvisioning.name,
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AWSIoTThingsRegistration",
 * });
 * const devicePolicy = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["iot:Subscribe"],
 *         resources: ["*"],
 *     }],
 * });
 * const devicePolicyPolicy = new aws.iot.Policy("device_policy", {
 *     name: "DevicePolicy",
 *     policy: devicePolicy.then(devicePolicy => devicePolicy.json),
 * });
 * const fleet = new aws.iot.ProvisioningTemplate("fleet", {
 *     name: "FleetTemplate",
 *     description: "My provisioning template",
 *     provisioningRoleArn: iotFleetProvisioning.arn,
 *     enabled: true,
 *     templateBody: pulumi.jsonStringify({
 *         Parameters: {
 *             SerialNumber: {
 *                 Type: "String",
 *             },
 *         },
 *         Resources: {
 *             certificate: {
 *                 Properties: {
 *                     CertificateId: {
 *                         Ref: "AWS::IoT::Certificate::Id",
 *                     },
 *                     Status: "Active",
 *                 },
 *                 Type: "AWS::IoT::Certificate",
 *             },
 *             policy: {
 *                 Properties: {
 *                     PolicyName: devicePolicyPolicy.name,
 *                 },
 *                 Type: "AWS::IoT::Policy",
 *             },
 *         },
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IoT fleet provisioning templates using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/provisioningTemplate:ProvisioningTemplate fleet FleetProvisioningTemplate
 * ```
 */
export declare class ProvisioningTemplate extends pulumi.CustomResource {
    /**
     * Get an existing ProvisioningTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProvisioningTemplateState, opts?: pulumi.CustomResourceOptions): ProvisioningTemplate;
    /**
     * Returns true if the given object is an instance of ProvisioningTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProvisioningTemplate;
    /**
     * The ARN that identifies the provisioning template.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The default version of the fleet provisioning template.
     */
    readonly defaultVersionId: pulumi.Output<number>;
    /**
     * The description of the fleet provisioning template.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * True to enable the fleet provisioning template, otherwise false.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * The name of the fleet provisioning template.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a pre-provisioning hook template. Details below.
     */
    readonly preProvisioningHook: pulumi.Output<outputs.iot.ProvisioningTemplatePreProvisioningHook | undefined>;
    /**
     * The role ARN for the role associated with the fleet provisioning template. This IoT role grants permission to provision a device.
     */
    readonly provisioningRoleArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * The JSON formatted contents of the fleet provisioning template.
     */
    readonly templateBody: pulumi.Output<string>;
    /**
     * The type you define in a provisioning template.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ProvisioningTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProvisioningTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProvisioningTemplate resources.
 */
export interface ProvisioningTemplateState {
    /**
     * The ARN that identifies the provisioning template.
     */
    arn?: pulumi.Input<string>;
    /**
     * The default version of the fleet provisioning template.
     */
    defaultVersionId?: pulumi.Input<number>;
    /**
     * The description of the fleet provisioning template.
     */
    description?: pulumi.Input<string>;
    /**
     * True to enable the fleet provisioning template, otherwise false.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * The name of the fleet provisioning template.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a pre-provisioning hook template. Details below.
     */
    preProvisioningHook?: pulumi.Input<inputs.iot.ProvisioningTemplatePreProvisioningHook>;
    /**
     * The role ARN for the role associated with the fleet provisioning template. This IoT role grants permission to provision a device.
     */
    provisioningRoleArn?: pulumi.Input<string>;
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
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The JSON formatted contents of the fleet provisioning template.
     */
    templateBody?: pulumi.Input<string>;
    /**
     * The type you define in a provisioning template.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ProvisioningTemplate resource.
 */
export interface ProvisioningTemplateArgs {
    /**
     * The description of the fleet provisioning template.
     */
    description?: pulumi.Input<string>;
    /**
     * True to enable the fleet provisioning template, otherwise false.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * The name of the fleet provisioning template.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a pre-provisioning hook template. Details below.
     */
    preProvisioningHook?: pulumi.Input<inputs.iot.ProvisioningTemplatePreProvisioningHook>;
    /**
     * The role ARN for the role associated with the fleet provisioning template. This IoT role grants permission to provision a device.
     */
    provisioningRoleArn: pulumi.Input<string>;
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
     * The JSON formatted contents of the fleet provisioning template.
     */
    templateBody: pulumi.Input<string>;
    /**
     * The type you define in a provisioning template.
     */
    type?: pulumi.Input<string>;
}
