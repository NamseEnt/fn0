import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage a GuardDuty IPSet.
 *
 * > **Note:** Currently in GuardDuty, users from member accounts cannot upload and further manage IPSets. IPSets that are uploaded by the primary account are imposed on GuardDuty functionality in its member accounts. See the [GuardDuty API Documentation](https://docs.aws.amazon.com/guardduty/latest/ug/create-ip-set.html)
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const primary = new aws.guardduty.Detector("primary", {enable: true});
 * const bucket = new aws.s3.Bucket("bucket", {});
 * const myIPSet = new aws.s3.BucketObjectv2("MyIPSet", {
 *     content: "10.0.0.0/8\n",
 *     bucket: bucket.id,
 *     key: "MyIPSet",
 * });
 * const example = new aws.guardduty.IPSet("example", {
 *     activate: true,
 *     detectorId: primary.id,
 *     format: "TXT",
 *     location: pulumi.interpolate`https://s3.amazonaws.com/${myIPSet.bucket}/${myIPSet.key}`,
 *     name: "MyIPSet",
 * });
 * const bucketAcl = new aws.s3.BucketAcl("bucket_acl", {
 *     bucket: bucket.id,
 *     acl: "private",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import GuardDuty IPSet using the primary GuardDuty detector ID and IPSet ID. For example:
 *
 * ```sh
 * $ pulumi import aws:guardduty/iPSet:IPSet MyIPSet 00b00fd5aecc0ab60a708659477e9617:123456789012
 * ```
 */
export declare class IPSet extends pulumi.CustomResource {
    /**
     * Get an existing IPSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IPSetState, opts?: pulumi.CustomResourceOptions): IPSet;
    /**
     * Returns true if the given object is an instance of IPSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IPSet;
    /**
     * Specifies whether GuardDuty is to start using the uploaded IPSet.
     */
    readonly activate: pulumi.Output<boolean>;
    /**
     * Amazon Resource Name (ARN) of the GuardDuty IPSet.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The detector ID of the GuardDuty.
     */
    readonly detectorId: pulumi.Output<string>;
    /**
     * The format of the file that contains the IPSet. Valid values: `TXT` | `STIX` | `OTX_CSV` | `ALIEN_VAULT` | `PROOF_POINT` | `FIRE_EYE`
     */
    readonly format: pulumi.Output<string>;
    /**
     * The URI of the file that contains the IPSet.
     */
    readonly location: pulumi.Output<string>;
    /**
     * The friendly name to identify the IPSet.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a IPSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IPSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IPSet resources.
 */
export interface IPSetState {
    /**
     * Specifies whether GuardDuty is to start using the uploaded IPSet.
     */
    activate?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the GuardDuty IPSet.
     */
    arn?: pulumi.Input<string>;
    /**
     * The detector ID of the GuardDuty.
     */
    detectorId?: pulumi.Input<string>;
    /**
     * The format of the file that contains the IPSet. Valid values: `TXT` | `STIX` | `OTX_CSV` | `ALIEN_VAULT` | `PROOF_POINT` | `FIRE_EYE`
     */
    format?: pulumi.Input<string>;
    /**
     * The URI of the file that contains the IPSet.
     */
    location?: pulumi.Input<string>;
    /**
     * The friendly name to identify the IPSet.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a IPSet resource.
 */
export interface IPSetArgs {
    /**
     * Specifies whether GuardDuty is to start using the uploaded IPSet.
     */
    activate: pulumi.Input<boolean>;
    /**
     * The detector ID of the GuardDuty.
     */
    detectorId: pulumi.Input<string>;
    /**
     * The format of the file that contains the IPSet. Valid values: `TXT` | `STIX` | `OTX_CSV` | `ALIEN_VAULT` | `PROOF_POINT` | `FIRE_EYE`
     */
    format: pulumi.Input<string>;
    /**
     * The URI of the file that contains the IPSet.
     */
    location: pulumi.Input<string>;
    /**
     * The friendly name to identify the IPSet.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
