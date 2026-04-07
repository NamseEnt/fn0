/**
 * An error resulting from the invocation of a Pulumi command.
 *
 * @alpha
 */
export declare class CommandError extends Error {
    private commandResult;
}
/**
 * An error thrown when attempting to update a stack that is already being
 * updated.
 */
export declare class ConcurrentUpdateError extends CommandError {
}
/**
 * An error thrown when attempting to select a stack that does not exist.
 */
export declare class StackNotFoundError extends CommandError {
}
/**
 * An error thrown when attempting to create a stack that already exists.
 */
export declare class StackAlreadyExistsError extends CommandError {
}
