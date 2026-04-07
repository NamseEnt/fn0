/**
 * {@link MockResourceArgs} is used to construct a new resource mock.
 */
export interface MockResourceArgs {
    /**
     * The token that indicates which resource type is being constructed. This
     * token is of the form "package:module:type".
     */
    type: string;
    /**
     * The logical name of the resource instance.
     */
    name: string;
    /**
     * The inputs for the resource.
     */
    inputs: any;
    /**
     * If provided, the identifier of the provider instance being used to manage
     * this resource.
     */
    provider?: string;
    /**
     * Specifies whether or not the resource is Custom (i.e. managed by a
     * resource provider).
     */
    custom?: boolean;
    /**
     * If provided, the physical identifier of an existing resource to read or
     * import.
     */
    id?: string;
}
/**
 * {@link MockResourceResult} is the result of a new resource mock, returning a
 * physical identifier and the output properties for the resource being
 * constructed.
 */
export declare type MockResourceResult = {
    id: string | undefined;
    state: Record<string, any>;
};
/**
 * {@link MockResourceArgs} is used to construct call mocks.
 */
export interface MockCallArgs {
    /**
     * The token that indicates which function is being called. This token is of
     * the form "package:module:function".
     */
    token: string;
    /**
     * The arguments provided to the function call.
     */
    inputs: any;
    /**
     * If provided, the identifier of the provider instance being used to make
     * the call.
     */
    provider?: string;
}
/**
 * {@link MockCallResult} is the result of a call mock.
 */
export declare type MockCallResult = Record<string, any>;
/**
 * {@link Mocks} allows implementations to replace operations normally
 * implemented by the Pulumi engine with their own implementations. This can be
 * used during testing to ensure that calls to provider functions and resource
 * constructors return predictable values.
 */
export interface Mocks {
    /**
     * Mocks provider-implemented function calls (e.g. `aws.get_availability_zones`).
     *
     * @param args MockCallArgs
     */
    call(args: MockCallArgs): MockCallResult | Promise<MockCallResult>;
    /**
     * Mocks resource construction calls. This function should return the
     * physical identifier and the output properties for the resource being
     * constructed.
     *
     * @param args MockResourceArgs
     */
    newResource(args: MockResourceArgs): MockResourceResult | Promise<MockResourceResult>;
}
export declare class MockMonitor {
    readonly mocks: Mocks;
    readonly resources: Map<string, {
        urn: string;
        id: string | null;
        state: any;
    }>;
    constructor(mocks: Mocks);
    private newUrn;
    invoke(req: any, callback: (err: any, innerResponse: any) => void): Promise<void>;
    readResource(req: any, callback: (err: any, innterResponse: any) => void): Promise<void>;
    registerResource(req: any, callback: (err: any, innerResponse: any) => void): Promise<void>;
    registerResourceOutputs(req: any, callback: (err: any, innerResponse: any) => void): void;
    supportsFeature(req: any, callback: (err: any, innerResponse: any) => void): void;
    registerPackage(req: any, callback: (err: any, innerResponse: any) => void): void;
}
/**
 * Configures the Pulumi runtime to use the given mocks for testing.
 *
 * @param mocks
 *  The mocks to use for calls to provider functions and resource construction.
 * @param project
 *  If provided, the name of the Pulumi project. Defaults to "project".
 * @param stack
 *  If provided, the name of the Pulumi stack. Defaults to "stack".
 * @param preview
 *  If provided, indicates whether or not the program is running a preview. Defaults to false.
 * @param organization
 *  If provided, the name of the Pulumi organization. Defaults to nothing.
 */
export declare function setMocks(mocks: Mocks, project?: string, stack?: string, preview?: boolean, organization?: string): Promise<void>;
