import * as pulumi from "@pulumi/pulumi";

export interface fn0Args {
  value: pulumi.Input<string>;
  count: pulumi.Input<number>;
}

class fn0 extends pulumi.ComponentResource {
  constructor(
    name: string,
    args: fn0Args,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:fn0", name, {}, opts);
  }
}
