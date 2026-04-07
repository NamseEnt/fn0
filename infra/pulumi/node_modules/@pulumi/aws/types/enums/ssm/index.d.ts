export declare const ParameterType: {
    readonly String: "String";
    readonly StringList: "StringList";
    readonly SecureString: "SecureString";
};
export type ParameterType = (typeof ParameterType)[keyof typeof ParameterType];
