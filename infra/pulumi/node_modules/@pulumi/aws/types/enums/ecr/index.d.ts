export declare const LifecyclePolicyActionType: {
    readonly Expire: "expire";
};
/**
 * The type of action to take.
 */
export type LifecyclePolicyActionType = (typeof LifecyclePolicyActionType)[keyof typeof LifecyclePolicyActionType];
export declare const LifecyclePolicyCountType: {
    readonly ImageCountMoreThan: "imageCountMoreThan";
    readonly SinceImagePushed: "sinceImagePushed";
};
/**
 * The type of count to perform.
 */
export type LifecyclePolicyCountType = (typeof LifecyclePolicyCountType)[keyof typeof LifecyclePolicyCountType];
export declare const LifecyclePolicyTagStatus: {
    readonly Tagged: "tagged";
    readonly Untagged: "untagged";
    readonly Any: "any";
};
/**
 * The tag status of the image.
 */
export type LifecyclePolicyTagStatus = (typeof LifecyclePolicyTagStatus)[keyof typeof LifecyclePolicyTagStatus];
