import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Cognito User Pool Domain resource.
 *
 * ## Example Usage
 *
 * ### Amazon Cognito domain
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {name: "example-pool"});
 * const main = new aws.cognito.UserPoolDomain("main", {
 *     domain: "example-domain",
 *     userPoolId: example.id,
 * });
 * ```
 *
 * ### Custom Cognito domain
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleUserPool = new aws.cognito.UserPool("example", {name: "example-pool"});
 * const main = new aws.cognito.UserPoolDomain("main", {
 *     domain: "auth.example.com",
 *     certificateArn: cert.arn,
 *     userPoolId: exampleUserPool.id,
 * });
 * const example = aws.route53.getZone({
 *     name: "example.com",
 * });
 * const auth_cognito_A = new aws.route53.Record("auth-cognito-A", {
 *     name: main.domain,
 *     type: aws.route53.RecordType.A,
 *     zoneId: example.then(example => example.zoneId),
 *     aliases: [{
 *         evaluateTargetHealth: false,
 *         name: main.cloudfrontDistribution,
 *         zoneId: main.cloudfrontDistributionZoneId,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cognito User Pool Domains using the `domain`. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/userPoolDomain:UserPoolDomain main auth.example.org
 * ```
 */
export declare class UserPoolDomain extends pulumi.CustomResource {
    /**
     * Get an existing UserPoolDomain resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserPoolDomainState, opts?: pulumi.CustomResourceOptions): UserPoolDomain;
    /**
     * Returns true if the given object is an instance of UserPoolDomain.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserPoolDomain;
    /**
     * The AWS account ID for the user pool owner.
     */
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The ARN of an ISSUED ACM certificate in us-east-1 for a custom domain.
     */
    readonly certificateArn: pulumi.Output<string | undefined>;
    /**
     * The Amazon CloudFront endpoint (e.g. `dpp0gtxikpq3y.cloudfront.net`) that you use as the target of the alias that you set up with your Domain Name Service (DNS) provider.
     */
    readonly cloudfrontDistribution: pulumi.Output<string>;
    /**
     * The URL of the CloudFront distribution. This is required to generate the ALIAS `aws.route53.Record`
     */
    readonly cloudfrontDistributionArn: pulumi.Output<string>;
    /**
     * The Route 53 hosted zone ID of the CloudFront distribution.
     */
    readonly cloudfrontDistributionZoneId: pulumi.Output<string>;
    /**
     * For custom domains, this is the fully-qualified domain name, such as auth.example.com. For Amazon Cognito prefix domains, this is the prefix alone, such as auth.
     */
    readonly domain: pulumi.Output<string>;
    /**
     * A version number that indicates the state of managed login for your domain. Valid values: `1` for hosted UI (classic), `2` for the newer managed login with the branding designer.
     */
    readonly managedLoginVersion: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The S3 bucket where the static files for this domain are stored.
     */
    readonly s3Bucket: pulumi.Output<string>;
    /**
     * The user pool ID.
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * The app version.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a UserPoolDomain resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserPoolDomainArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserPoolDomain resources.
 */
export interface UserPoolDomainState {
    /**
     * The AWS account ID for the user pool owner.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * The ARN of an ISSUED ACM certificate in us-east-1 for a custom domain.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * The Amazon CloudFront endpoint (e.g. `dpp0gtxikpq3y.cloudfront.net`) that you use as the target of the alias that you set up with your Domain Name Service (DNS) provider.
     */
    cloudfrontDistribution?: pulumi.Input<string>;
    /**
     * The URL of the CloudFront distribution. This is required to generate the ALIAS `aws.route53.Record`
     */
    cloudfrontDistributionArn?: pulumi.Input<string>;
    /**
     * The Route 53 hosted zone ID of the CloudFront distribution.
     */
    cloudfrontDistributionZoneId?: pulumi.Input<string>;
    /**
     * For custom domains, this is the fully-qualified domain name, such as auth.example.com. For Amazon Cognito prefix domains, this is the prefix alone, such as auth.
     */
    domain?: pulumi.Input<string>;
    /**
     * A version number that indicates the state of managed login for your domain. Valid values: `1` for hosted UI (classic), `2` for the newer managed login with the branding designer.
     */
    managedLoginVersion?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The S3 bucket where the static files for this domain are stored.
     */
    s3Bucket?: pulumi.Input<string>;
    /**
     * The user pool ID.
     */
    userPoolId?: pulumi.Input<string>;
    /**
     * The app version.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserPoolDomain resource.
 */
export interface UserPoolDomainArgs {
    /**
     * The ARN of an ISSUED ACM certificate in us-east-1 for a custom domain.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * For custom domains, this is the fully-qualified domain name, such as auth.example.com. For Amazon Cognito prefix domains, this is the prefix alone, such as auth.
     */
    domain: pulumi.Input<string>;
    /**
     * A version number that indicates the state of managed login for your domain. Valid values: `1` for hosted UI (classic), `2` for the newer managed login with the branding designer.
     */
    managedLoginVersion?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user pool ID.
     */
    userPoolId: pulumi.Input<string>;
}
