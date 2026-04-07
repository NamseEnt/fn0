import { Resource } from "./resource";
/**
 * {@link RunError} can be used for terminating a program abruptly, but
 * resulting in a clean exit rather than the usual verbose unhandled error logic
 * which emits the source program text and complete stack trace. This type
 * should be rarely used. Ideally {@link ResourceError} should always be used so
 * that as many errors as possible can be associated with a resource.
 */
export declare class RunError extends Error {
    /**
     * Returns true if the given object is a {@link RunError}. This is designed
     * to work even when multiple copies of the Pulumi SDK have been loaded into
     * the same process.
     */
    static isInstance(obj: any): obj is RunError;
}
/**
 * {@link ResourceError} can be used for terminating a program abruptly,
 * specifically associating the problem with a {@link Resource}. Depending on
 * the nature of the problem, clients can choose whether or not the call stack
 * should be hidden as well. This should be very rare, and would only indicate
 * that presenting the stack to the user would not be useful/be detrimental.
 */
export declare class ResourceError extends Error {
    resource: Resource | undefined;
    hideStack?: boolean | undefined;
    /**
     * Returns true if the given object is a {@link ResourceError}. This is
     * designed to work even when multiple copies of the Pulumi SDK have been
     * loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceError;
    constructor(message: string, resource: Resource | undefined, hideStack?: boolean | undefined);
}
export declare function isGrpcError(err: Error): boolean;
/**
 * {@link InputPropertyError} is an error type that is used to indicate that a
 * an input property has failed validation.
 *
 * The engine will pretty print this error for the user.
 */
export declare class InputPropertyError extends Error {
    propertyPath: string;
    reason: string;
    constructor(args: InputPropertyErrorDetails);
    static isInstance(obj: any): obj is InputPropertyError;
}
export interface InputPropertyErrorDetails {
    propertyPath: string;
    reason: string;
}
interface InputPropertiesErrorArgs {
    message: string;
    errors: Array<InputPropertyErrorDetails>;
}
/**
 * {@link InputPropertiesError} is an error type that is used to indicate that
 * multiple input properties have failed validation.
 *
 * The engine will pretty print this error for the user.
 */
export declare class InputPropertiesError extends Error {
    message: string;
    errors: Array<InputPropertyErrorDetails>;
    constructor(args: InputPropertiesErrorArgs);
    static isInstance(obj: any): obj is InputPropertiesError;
}
export {};
