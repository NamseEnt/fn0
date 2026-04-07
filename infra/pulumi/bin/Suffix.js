"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Suffix = void 0;
const pulumi = require("@pulumi/pulumi");
const random = require("@pulumi/random");
class Suffix extends pulumi.ComponentResource {
    constructor(name, args, opts) {
        super("pkg:index:suffix", name, args, opts);
        const randomString = new random.RandomString("suffix", {
            length: 8,
            special: false,
            upper: false,
        }, { parent: this });
        this.result = randomString.result;
    }
}
exports.Suffix = Suffix;
//# sourceMappingURL=Suffix.js.map