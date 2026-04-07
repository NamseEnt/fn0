/**
 * Returns the current organization name.
 */
export declare function getOrganization(): string;
/**
 * Returns the current project name. Throws an exception if none is registered.
 */
export declare function getProject(): string;
/**
 * Returns the current stack name. Throws an exception if none is registered.
 */
export declare function getStack(): string;
/**
 * Returns the root directory of the current Pulumi project.
 */
export declare function getRootDirectory(): string;
