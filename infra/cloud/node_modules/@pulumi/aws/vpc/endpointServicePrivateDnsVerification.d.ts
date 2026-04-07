import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC (Virtual Private Cloud) Endpoint Service Private DNS Verification.
 * This resource begins the verification process by calling the [`StartVpcEndpointServicePrivateDnsVerification`](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_StartVpcEndpointServicePrivateDnsVerification.html) API.
 * The service provider should add a record to the DNS server _before_ creating this resource.
 *
 * For additional details, refer to the AWS documentation on [managing VPC endpoint service DNS names](https://docs.aws.amazon.com/vpc/latest/privatelink/manage-dns-names.html).
 *
 * > Destruction of this resource will not stop the verification process, only remove the resource from state.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpc.EndpointServicePrivateDnsVerification("example", {serviceId: exampleAwsVpcEndpointService.id});
 * ```
 *
 * ## Import
 *
 * You cannot import this resource.
 */
export declare class EndpointServicePrivateDnsVerification extends pulumi.CustomResource {
    /**
     * Get an existing EndpointServicePrivateDnsVerification resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointServicePrivateDnsVerificationState, opts?: pulumi.CustomResourceOptions): EndpointServicePrivateDnsVerification;
    /**
     * Returns true if the given object is an instance of EndpointServicePrivateDnsVerification.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EndpointServicePrivateDnsVerification;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the endpoint service.
     *
     * The following arguments are optional:
     */
    readonly serviceId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.vpc.EndpointServicePrivateDnsVerificationTimeouts | undefined>;
    /**
     * Whether to wait until the endpoint service returns a `Verified` status for the configured private DNS name.
     */
    readonly waitForVerification: pulumi.Output<boolean | undefined>;
    /**
     * Create a EndpointServicePrivateDnsVerification resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointServicePrivateDnsVerificationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EndpointServicePrivateDnsVerification resources.
 */
export interface EndpointServicePrivateDnsVerificationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the endpoint service.
     *
     * The following arguments are optional:
     */
    serviceId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.EndpointServicePrivateDnsVerificationTimeouts>;
    /**
     * Whether to wait until the endpoint service returns a `Verified` status for the configured private DNS name.
     */
    waitForVerification?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a EndpointServicePrivateDnsVerification resource.
 */
export interface EndpointServicePrivateDnsVerificationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the endpoint service.
     *
     * The following arguments are optional:
     */
    serviceId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.EndpointServicePrivateDnsVerificationTimeouts>;
    /**
     * Whether to wait until the endpoint service returns a `Verified` status for the configured private DNS name.
     */
    waitForVerification?: pulumi.Input<boolean>;
}
