import * as pulumi from "@pulumi/pulumi";
import * as random from "@pulumi/random";

export interface SuffixArgs {}

export class Suffix extends pulumi.ComponentResource {
  result: pulumi.Output<string>;

  constructor(
    name: string,
    args: SuffixArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:suffix", name, args, opts);

    const randomString = new random.RandomString(
      "suffix",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    );

    this.result = randomString.result;
  }
}
