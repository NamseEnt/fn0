import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Route 53 Profiles.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getProfilesProfiles({});
 * ```
 */
export declare function getProfilesProfiles(args?: GetProfilesProfilesArgs, opts?: pulumi.InvokeOptions): Promise<GetProfilesProfilesResult>;
/**
 * A collection of arguments for invoking getProfilesProfiles.
 */
export interface GetProfilesProfilesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getProfilesProfiles.
 */
export interface GetProfilesProfilesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of Profiles.
     */
    readonly profiles: outputs.route53.GetProfilesProfilesProfile[];
    readonly region: string;
}
/**
 * Data source for managing an AWS Route 53 Profiles.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getProfilesProfiles({});
 * ```
 */
export declare function getProfilesProfilesOutput(args?: GetProfilesProfilesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetProfilesProfilesResult>;
/**
 * A collection of arguments for invoking getProfilesProfiles.
 */
export interface GetProfilesProfilesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
