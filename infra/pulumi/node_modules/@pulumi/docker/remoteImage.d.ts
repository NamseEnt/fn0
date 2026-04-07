import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
export declare class RemoteImage extends pulumi.CustomResource {
    /**
     * Get an existing RemoteImage resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RemoteImageState, opts?: pulumi.CustomResourceOptions): RemoteImage;
    /**
     * Returns true if the given object is an instance of RemoteImage.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RemoteImage;
    readonly build: pulumi.Output<outputs.RemoteImageBuild | undefined>;
    /**
     * If true, then the image is removed forcibly when the resource is destroyed.
     */
    readonly forceRemove: pulumi.Output<boolean | undefined>;
    /**
     * The ID of the image (as seen when executing `docker inspect` on the image). Can be used to reference the image via its ID in other resources.
     */
    readonly imageId: pulumi.Output<string>;
    /**
     * If true, then the Docker image won't be deleted on destroy operation. If this is false, it will delete the image from the docker local storage on destroy operation.
     */
    readonly keepLocally: pulumi.Output<boolean | undefined>;
    /**
     * The name of the Docker image, including any tags or SHA256 repo digests.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The platform to use when pulling the image. Defaults to the platform of the current machine.
     */
    readonly platform: pulumi.Output<string | undefined>;
    /**
     * List of values which cause an image pull when changed. This is used to store the image digest from the registry when using the docker*registry*image.
     */
    readonly pullTriggers: pulumi.Output<string[] | undefined>;
    /**
     * The image sha256 digest in the form of `repo[:tag]@sha256:<hash>`. This may not be populated when building an image, because it is read from the local Docker client and so may be available only when the image was either pulled from the repo or pushed to the repo (perhaps using `docker.RegistryImage`) in a previous run.
     */
    readonly repoDigest: pulumi.Output<string>;
    /**
     * A map of arbitrary strings that, when changed, will force the `docker.RemoteImage` resource to be replaced. This can be used to rebuild an image when contents of source code folders change
     */
    readonly triggers: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a RemoteImage resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RemoteImageArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RemoteImage resources.
 */
export interface RemoteImageState {
    build?: pulumi.Input<inputs.RemoteImageBuild>;
    /**
     * If true, then the image is removed forcibly when the resource is destroyed.
     */
    forceRemove?: pulumi.Input<boolean>;
    /**
     * The ID of the image (as seen when executing `docker inspect` on the image). Can be used to reference the image via its ID in other resources.
     */
    imageId?: pulumi.Input<string>;
    /**
     * If true, then the Docker image won't be deleted on destroy operation. If this is false, it will delete the image from the docker local storage on destroy operation.
     */
    keepLocally?: pulumi.Input<boolean>;
    /**
     * The name of the Docker image, including any tags or SHA256 repo digests.
     */
    name?: pulumi.Input<string>;
    /**
     * The platform to use when pulling the image. Defaults to the platform of the current machine.
     */
    platform?: pulumi.Input<string>;
    /**
     * List of values which cause an image pull when changed. This is used to store the image digest from the registry when using the docker*registry*image.
     */
    pullTriggers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The image sha256 digest in the form of `repo[:tag]@sha256:<hash>`. This may not be populated when building an image, because it is read from the local Docker client and so may be available only when the image was either pulled from the repo or pushed to the repo (perhaps using `docker.RegistryImage`) in a previous run.
     */
    repoDigest?: pulumi.Input<string>;
    /**
     * A map of arbitrary strings that, when changed, will force the `docker.RemoteImage` resource to be replaced. This can be used to rebuild an image when contents of source code folders change
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a RemoteImage resource.
 */
export interface RemoteImageArgs {
    build?: pulumi.Input<inputs.RemoteImageBuild>;
    /**
     * If true, then the image is removed forcibly when the resource is destroyed.
     */
    forceRemove?: pulumi.Input<boolean>;
    /**
     * If true, then the Docker image won't be deleted on destroy operation. If this is false, it will delete the image from the docker local storage on destroy operation.
     */
    keepLocally?: pulumi.Input<boolean>;
    /**
     * The name of the Docker image, including any tags or SHA256 repo digests.
     */
    name: pulumi.Input<string>;
    /**
     * The platform to use when pulling the image. Defaults to the platform of the current machine.
     */
    platform?: pulumi.Input<string>;
    /**
     * List of values which cause an image pull when changed. This is used to store the image digest from the registry when using the docker*registry*image.
     */
    pullTriggers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of arbitrary strings that, when changed, will force the `docker.RemoteImage` resource to be replaced. This can be used to rebuild an image when contents of source code folders change
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
