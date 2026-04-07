import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon Customer Profiles Profile.
 * See the [Create Profile](https://docs.aws.amazon.com/customerprofiles/latest/APIReference/API_CreateProfile.html) for more information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.customerprofiles.Domain("example", {domainName: "example"});
 * const exampleProfile = new aws.customerprofiles.Profile("example", {domainName: example.domainName});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Customer Profiles Profile using the resource `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:customerprofiles/profile:Profile example domain-name/5f2f473dfbe841eb8d05cfc2a4c926df
 * ```
 */
export declare class Profile extends pulumi.CustomResource {
    /**
     * Get an existing Profile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProfileState, opts?: pulumi.CustomResourceOptions): Profile;
    /**
     * Returns true if the given object is an instance of Profile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Profile;
    /**
     * A unique account number that you have given to the customer.
     */
    readonly accountNumber: pulumi.Output<string | undefined>;
    /**
     * Any additional information relevant to the customer’s profile.
     */
    readonly additionalInformation: pulumi.Output<string | undefined>;
    /**
     * A block that specifies a generic address associated with the customer that is not mailing, shipping, or billing. Documented below.
     */
    readonly address: pulumi.Output<outputs.customerprofiles.ProfileAddress | undefined>;
    /**
     * A key value pair of attributes of a customer profile.
     */
    readonly attributes: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A block that specifies the customer’s billing address. Documented below.
     */
    readonly billingAddress: pulumi.Output<outputs.customerprofiles.ProfileBillingAddress | undefined>;
    /**
     * The customer’s birth date.
     */
    readonly birthDate: pulumi.Output<string | undefined>;
    /**
     * The customer’s business email address.
     */
    readonly businessEmailAddress: pulumi.Output<string | undefined>;
    /**
     * The name of the customer’s business.
     */
    readonly businessName: pulumi.Output<string | undefined>;
    /**
     * The customer’s business phone number.
     */
    readonly businessPhoneNumber: pulumi.Output<string | undefined>;
    /**
     * The name of your Customer Profile domain. It must be unique for your AWS account.
     *
     * The following arguments are optional:
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The customer’s email address, which has not been specified as a personal or business address.
     */
    readonly emailAddress: pulumi.Output<string | undefined>;
    /**
     * The customer’s first name.
     */
    readonly firstName: pulumi.Output<string | undefined>;
    /**
     * The gender with which the customer identifies.
     */
    readonly genderString: pulumi.Output<string | undefined>;
    /**
     * The customer’s home phone number.
     */
    readonly homePhoneNumber: pulumi.Output<string | undefined>;
    /**
     * The customer’s last name.
     */
    readonly lastName: pulumi.Output<string | undefined>;
    /**
     * A block that specifies the customer’s mailing address. Documented below.
     */
    readonly mailingAddress: pulumi.Output<outputs.customerprofiles.ProfileMailingAddress | undefined>;
    /**
     * The customer’s middle name.
     */
    readonly middleName: pulumi.Output<string | undefined>;
    /**
     * The customer’s mobile phone number.
     */
    readonly mobilePhoneNumber: pulumi.Output<string | undefined>;
    /**
     * The type of profile used to describe the customer.
     */
    readonly partyTypeString: pulumi.Output<string | undefined>;
    /**
     * The customer’s personal email address.
     */
    readonly personalEmailAddress: pulumi.Output<string | undefined>;
    /**
     * The customer’s phone number, which has not been specified as a mobile, home, or business number.
     */
    readonly phoneNumber: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A block that specifies the customer’s shipping address. Documented below.
     */
    readonly shippingAddress: pulumi.Output<outputs.customerprofiles.ProfileShippingAddress | undefined>;
    /**
     * Create a Profile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Profile resources.
 */
export interface ProfileState {
    /**
     * A unique account number that you have given to the customer.
     */
    accountNumber?: pulumi.Input<string>;
    /**
     * Any additional information relevant to the customer’s profile.
     */
    additionalInformation?: pulumi.Input<string>;
    /**
     * A block that specifies a generic address associated with the customer that is not mailing, shipping, or billing. Documented below.
     */
    address?: pulumi.Input<inputs.customerprofiles.ProfileAddress>;
    /**
     * A key value pair of attributes of a customer profile.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A block that specifies the customer’s billing address. Documented below.
     */
    billingAddress?: pulumi.Input<inputs.customerprofiles.ProfileBillingAddress>;
    /**
     * The customer’s birth date.
     */
    birthDate?: pulumi.Input<string>;
    /**
     * The customer’s business email address.
     */
    businessEmailAddress?: pulumi.Input<string>;
    /**
     * The name of the customer’s business.
     */
    businessName?: pulumi.Input<string>;
    /**
     * The customer’s business phone number.
     */
    businessPhoneNumber?: pulumi.Input<string>;
    /**
     * The name of your Customer Profile domain. It must be unique for your AWS account.
     *
     * The following arguments are optional:
     */
    domainName?: pulumi.Input<string>;
    /**
     * The customer’s email address, which has not been specified as a personal or business address.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * The customer’s first name.
     */
    firstName?: pulumi.Input<string>;
    /**
     * The gender with which the customer identifies.
     */
    genderString?: pulumi.Input<string>;
    /**
     * The customer’s home phone number.
     */
    homePhoneNumber?: pulumi.Input<string>;
    /**
     * The customer’s last name.
     */
    lastName?: pulumi.Input<string>;
    /**
     * A block that specifies the customer’s mailing address. Documented below.
     */
    mailingAddress?: pulumi.Input<inputs.customerprofiles.ProfileMailingAddress>;
    /**
     * The customer’s middle name.
     */
    middleName?: pulumi.Input<string>;
    /**
     * The customer’s mobile phone number.
     */
    mobilePhoneNumber?: pulumi.Input<string>;
    /**
     * The type of profile used to describe the customer.
     */
    partyTypeString?: pulumi.Input<string>;
    /**
     * The customer’s personal email address.
     */
    personalEmailAddress?: pulumi.Input<string>;
    /**
     * The customer’s phone number, which has not been specified as a mobile, home, or business number.
     */
    phoneNumber?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A block that specifies the customer’s shipping address. Documented below.
     */
    shippingAddress?: pulumi.Input<inputs.customerprofiles.ProfileShippingAddress>;
}
/**
 * The set of arguments for constructing a Profile resource.
 */
export interface ProfileArgs {
    /**
     * A unique account number that you have given to the customer.
     */
    accountNumber?: pulumi.Input<string>;
    /**
     * Any additional information relevant to the customer’s profile.
     */
    additionalInformation?: pulumi.Input<string>;
    /**
     * A block that specifies a generic address associated with the customer that is not mailing, shipping, or billing. Documented below.
     */
    address?: pulumi.Input<inputs.customerprofiles.ProfileAddress>;
    /**
     * A key value pair of attributes of a customer profile.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A block that specifies the customer’s billing address. Documented below.
     */
    billingAddress?: pulumi.Input<inputs.customerprofiles.ProfileBillingAddress>;
    /**
     * The customer’s birth date.
     */
    birthDate?: pulumi.Input<string>;
    /**
     * The customer’s business email address.
     */
    businessEmailAddress?: pulumi.Input<string>;
    /**
     * The name of the customer’s business.
     */
    businessName?: pulumi.Input<string>;
    /**
     * The customer’s business phone number.
     */
    businessPhoneNumber?: pulumi.Input<string>;
    /**
     * The name of your Customer Profile domain. It must be unique for your AWS account.
     *
     * The following arguments are optional:
     */
    domainName: pulumi.Input<string>;
    /**
     * The customer’s email address, which has not been specified as a personal or business address.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * The customer’s first name.
     */
    firstName?: pulumi.Input<string>;
    /**
     * The gender with which the customer identifies.
     */
    genderString?: pulumi.Input<string>;
    /**
     * The customer’s home phone number.
     */
    homePhoneNumber?: pulumi.Input<string>;
    /**
     * The customer’s last name.
     */
    lastName?: pulumi.Input<string>;
    /**
     * A block that specifies the customer’s mailing address. Documented below.
     */
    mailingAddress?: pulumi.Input<inputs.customerprofiles.ProfileMailingAddress>;
    /**
     * The customer’s middle name.
     */
    middleName?: pulumi.Input<string>;
    /**
     * The customer’s mobile phone number.
     */
    mobilePhoneNumber?: pulumi.Input<string>;
    /**
     * The type of profile used to describe the customer.
     */
    partyTypeString?: pulumi.Input<string>;
    /**
     * The customer’s personal email address.
     */
    personalEmailAddress?: pulumi.Input<string>;
    /**
     * The customer’s phone number, which has not been specified as a mobile, home, or business number.
     */
    phoneNumber?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A block that specifies the customer’s shipping address. Documented below.
     */
    shippingAddress?: pulumi.Input<inputs.customerprofiles.ProfileShippingAddress>;
}
