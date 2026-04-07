import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an AWS Certificate Manager Private Certificate Authorities Permission.
 * Currently, this is only required in order to allow the ACM service to automatically renew certificates issued by a PCA.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleCertificateAuthority = new aws.acmpca.CertificateAuthority("example", {certificateAuthorityConfiguration: {
 *     keyAlgorithm: "RSA_4096",
 *     signingAlgorithm: "SHA512WITHRSA",
 *     subject: {
 *         commonName: "example.com",
 *     },
 * }});
 * const example = new aws.acmpca.Permission("example", {
 *     certificateAuthorityArn: exampleCertificateAuthority.arn,
 *     actions: [
 *         "IssueCertificate",
 *         "GetCertificate",
 *         "ListPermissions",
 *     ],
 *     principal: "acm.amazonaws.com",
 * });
 * ```
 */
export declare class Permission extends pulumi.CustomResource {
    /**
     * Get an existing Permission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PermissionState, opts?: pulumi.CustomResourceOptions): Permission;
    /**
     * Returns true if the given object is an instance of Permission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Permission;
    /**
     * Actions that the specified AWS service principal can use. These include `IssueCertificate`, `GetCertificate`, and `ListPermissions`. Note that in order for ACM to automatically rotate certificates issued by a PCA, it must be granted permission on all 3 actions, as per the example above.
     */
    readonly actions: pulumi.Output<string[]>;
    /**
     * ARN of the CA that grants the permissions.
     */
    readonly certificateAuthorityArn: pulumi.Output<string>;
    /**
     * IAM policy that is associated with the permission.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * AWS service or identity that receives the permission. At this time, the only valid principal is `acm.amazonaws.com`.
     */
    readonly principal: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the calling account
     */
    readonly sourceAccount: pulumi.Output<string>;
    /**
     * Create a Permission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Permission resources.
 */
export interface PermissionState {
    /**
     * Actions that the specified AWS service principal can use. These include `IssueCertificate`, `GetCertificate`, and `ListPermissions`. Note that in order for ACM to automatically rotate certificates issued by a PCA, it must be granted permission on all 3 actions, as per the example above.
     */
    actions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the CA that grants the permissions.
     */
    certificateAuthorityArn?: pulumi.Input<string>;
    /**
     * IAM policy that is associated with the permission.
     */
    policy?: pulumi.Input<string>;
    /**
     * AWS service or identity that receives the permission. At this time, the only valid principal is `acm.amazonaws.com`.
     */
    principal?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the calling account
     */
    sourceAccount?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Permission resource.
 */
export interface PermissionArgs {
    /**
     * Actions that the specified AWS service principal can use. These include `IssueCertificate`, `GetCertificate`, and `ListPermissions`. Note that in order for ACM to automatically rotate certificates issued by a PCA, it must be granted permission on all 3 actions, as per the example above.
     */
    actions: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the CA that grants the permissions.
     */
    certificateAuthorityArn: pulumi.Input<string>;
    /**
     * AWS service or identity that receives the permission. At this time, the only valid principal is `acm.amazonaws.com`.
     */
    principal: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the calling account
     */
    sourceAccount?: pulumi.Input<string>;
}
