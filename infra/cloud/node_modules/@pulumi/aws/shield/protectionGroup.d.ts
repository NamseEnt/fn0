import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a grouping of protected resources so they can be handled as a collective.
 * This resource grouping improves the accuracy of detection and reduces false positives. For more information see
 * [Managing AWS Shield Advanced protection groups](https://docs.aws.amazon.com/waf/latest/developerguide/manage-protection-group.html)
 *
 * ## Example Usage
 *
 * ### Create protection group for all resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.shield.ProtectionGroup("example", {
 *     protectionGroupId: "example",
 *     aggregation: "MAX",
 *     pattern: "ALL",
 * });
 * ```
 *
 * ### Create protection group for arbitrary number of resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const example = new aws.ec2.Eip("example", {domain: "vpc"});
 * const exampleProtection = new aws.shield.Protection("example", {
 *     name: "example",
 *     resourceArn: pulumi.all([current, currentGetCallerIdentity, example.id]).apply(([current, currentGetCallerIdentity, id]) => `arn:aws:ec2:${current.region}:${currentGetCallerIdentity.accountId}:eip-allocation/${id}`),
 * });
 * const exampleProtectionGroup = new aws.shield.ProtectionGroup("example", {
 *     protectionGroupId: "example",
 *     aggregation: "MEAN",
 *     pattern: "ARBITRARY",
 *     members: [pulumi.all([current, currentGetCallerIdentity, example.id]).apply(([current, currentGetCallerIdentity, id]) => `arn:aws:ec2:${current.region}:${currentGetCallerIdentity.accountId}:eip-allocation/${id}`)],
 * }, {
 *     dependsOn: [exampleProtection],
 * });
 * ```
 *
 * ### Create protection group for a type of resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.shield.ProtectionGroup("example", {
 *     protectionGroupId: "example",
 *     aggregation: "SUM",
 *     pattern: "BY_RESOURCE_TYPE",
 *     resourceType: "ELASTIC_IP_ALLOCATION",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Shield protection group resources using their protection group id. For example:
 *
 * ```sh
 * $ pulumi import aws:shield/protectionGroup:ProtectionGroup example example
 * ```
 */
export declare class ProtectionGroup extends pulumi.CustomResource {
    /**
     * Get an existing ProtectionGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProtectionGroupState, opts?: pulumi.CustomResourceOptions): ProtectionGroup;
    /**
     * Returns true if the given object is an instance of ProtectionGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProtectionGroup;
    /**
     * Defines how AWS Shield combines resource data for the group in order to detect, mitigate, and report events.
     */
    readonly aggregation: pulumi.Output<string>;
    /**
     * The Amazon Resource Names (ARNs) of the resources to include in the protection group. You must set this when you set `pattern` to ARBITRARY and you must not set it for any other `pattern` setting.
     */
    readonly members: pulumi.Output<string[] | undefined>;
    /**
     * The criteria to use to choose the protected resources for inclusion in the group.
     */
    readonly pattern: pulumi.Output<string>;
    /**
     * The ARN (Amazon Resource Name) of the protection group.
     */
    readonly protectionGroupArn: pulumi.Output<string>;
    /**
     * The name of the protection group.
     */
    readonly protectionGroupId: pulumi.Output<string>;
    /**
     * The resource type to include in the protection group. You must set this when you set `pattern` to BY_RESOURCE_TYPE and you must not set it for any other `pattern` setting.
     */
    readonly resourceType: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ProtectionGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProtectionGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProtectionGroup resources.
 */
export interface ProtectionGroupState {
    /**
     * Defines how AWS Shield combines resource data for the group in order to detect, mitigate, and report events.
     */
    aggregation?: pulumi.Input<string>;
    /**
     * The Amazon Resource Names (ARNs) of the resources to include in the protection group. You must set this when you set `pattern` to ARBITRARY and you must not set it for any other `pattern` setting.
     */
    members?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The criteria to use to choose the protected resources for inclusion in the group.
     */
    pattern?: pulumi.Input<string>;
    /**
     * The ARN (Amazon Resource Name) of the protection group.
     */
    protectionGroupArn?: pulumi.Input<string>;
    /**
     * The name of the protection group.
     */
    protectionGroupId?: pulumi.Input<string>;
    /**
     * The resource type to include in the protection group. You must set this when you set `pattern` to BY_RESOURCE_TYPE and you must not set it for any other `pattern` setting.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a ProtectionGroup resource.
 */
export interface ProtectionGroupArgs {
    /**
     * Defines how AWS Shield combines resource data for the group in order to detect, mitigate, and report events.
     */
    aggregation: pulumi.Input<string>;
    /**
     * The Amazon Resource Names (ARNs) of the resources to include in the protection group. You must set this when you set `pattern` to ARBITRARY and you must not set it for any other `pattern` setting.
     */
    members?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The criteria to use to choose the protected resources for inclusion in the group.
     */
    pattern: pulumi.Input<string>;
    /**
     * The name of the protection group.
     */
    protectionGroupId: pulumi.Input<string>;
    /**
     * The resource type to include in the protection group. You must set this when you set `pattern` to BY_RESOURCE_TYPE and you must not set it for any other `pattern` setting.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
