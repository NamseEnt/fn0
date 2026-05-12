import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";

export interface EventBridgeCronTriggerArgs {
  controlUrl: pulumi.Input<string>;
  controlAdminToken: pulumi.Input<string>;
  awsRegion: pulumi.Input<string>;
  suffix: pulumi.Input<string>;
}

export class EventBridgeCronTrigger extends pulumi.ComponentResource {
  public readonly eventRuleArn: pulumi.Output<string>;
  public readonly apiDestinationArn: pulumi.Output<string>;
  public readonly connectionArn: pulumi.Output<string>;

  constructor(
    name: string,
    args: EventBridgeCronTriggerArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:event-bridge-cron-trigger", name, args, opts);

    const connection = new aws.cloudwatch.EventConnection(
      "connection",
      {
        region: args.awsRegion,
        name: pulumi.interpolate`fn0-control-cron-${args.suffix}`,
        authorizationType: "API_KEY",
        authParameters: {
          apiKey: {
            key: "Authorization",
            value: pulumi.interpolate`Bearer ${args.controlAdminToken}`,
          },
        },
      },
      { parent: this }
    );

    const apiDestination = new aws.cloudwatch.EventApiDestination(
      "api-destination",
      {
        region: args.awsRegion,
        name: pulumi.interpolate`fn0-control-cron-${args.suffix}`,
        connectionArn: connection.arn,
        httpMethod: "POST",
        invocationEndpoint: pulumi.interpolate`${args.controlUrl}/__forte_action/cron_on_tick`,
      },
      { parent: this }
    );

    const invokerRole = new aws.iam.Role(
      "invoker-role",
      {
        name: pulumi.interpolate`fn0-control-cron-invoker-${args.suffix}`,
        assumeRolePolicy: JSON.stringify({
          Version: "2012-10-17",
          Statement: [
            {
              Effect: "Allow",
              Principal: { Service: "events.amazonaws.com" },
              Action: "sts:AssumeRole",
            },
          ],
        }),
      },
      { parent: this }
    );

    new aws.iam.RolePolicy(
      "invoker-role-policy",
      {
        role: invokerRole.id,
        policy: apiDestination.arn.apply((arn) =>
          JSON.stringify({
            Version: "2012-10-17",
            Statement: [
              {
                Effect: "Allow",
                Action: "events:InvokeApiDestination",
                Resource: arn,
              },
            ],
          })
        ),
      },
      { parent: this }
    );

    const eventRule = new aws.cloudwatch.EventRule(
      "rule",
      {
        region: args.awsRegion,
        name: pulumi.interpolate`fn0-control-cron-${args.suffix}`,
        scheduleExpression: "rate(1 minute)",
        state: "ENABLED",
      },
      { parent: this }
    );

    new aws.cloudwatch.EventTarget(
      "target",
      {
        region: args.awsRegion,
        rule: eventRule.name,
        arn: apiDestination.arn,
        roleArn: invokerRole.arn,
        inputTransformer: {
          inputPaths: { time: "$.time" },
          inputTemplate: '{"scheduled_time": "<time>"}',
        },
        retryPolicy: {
          maximumRetryAttempts: 0,
          maximumEventAgeInSeconds: 60,
        },
      },
      { parent: this }
    );

    this.eventRuleArn = eventRule.arn;
    this.apiDestinationArn = apiDestination.arn;
    this.connectionArn = connection.arn;

    this.registerOutputs({
      eventRuleArn: this.eventRuleArn,
      apiDestinationArn: this.apiDestinationArn,
      connectionArn: this.connectionArn,
    });
  }
}
