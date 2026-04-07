"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TursoDatabaseToken = void 0;
const pulumi = require("@pulumi/pulumi");
class TursoDatabaseToken extends pulumi.dynamic.Resource {
    constructor(name, args, opts) {
        super(new TursoDatabaseTokenProvider(), name, { ...args, jwt: undefined }, opts);
    }
}
exports.TursoDatabaseToken = TursoDatabaseToken;
class TursoDatabaseTokenProvider {
    async create(inputs) {
        const config = new pulumi.Config("turso");
        const apiKey = config.require("apiToken");
        const response = await fetch(`https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/databases/${inputs.databaseName}/auth/tokens`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${apiKey}`,
            },
        });
        if (!response.ok) {
            const error = await response.text();
            throw new Error(`Failed to create database token: ${error}`);
        }
        const data = await response.json();
        return {
            id: `${inputs.databaseName}-token`,
            outs: {
                ...inputs,
                ...data,
            },
        };
    }
}
//# sourceMappingURL=dbToken.js.map