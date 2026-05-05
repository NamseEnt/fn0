import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";

export interface EventBridgeCronTriggerArgs {
  controlUrl: pulumi.Input<string>;
  controlAdminToken: pulumi.Input<string>;
  awsRegion: pulumi.Input<string>;
  suffix: pulumi.Input<string>;
}

export class EventBridgeCronTrigger extends pulumi.ComponentResource {
  public readonly scheduleArn: pulumi.Output<string>;
  public readonly apiDestinationArn: pulumi.Output<string>;
  public readonly connectionArn: pulumi.Output<string>;

  constructor(
    name: string,
    args: EventBridgeCronTriggerArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:event-bridge-cron-trigger", name, args, opts);

    const provider = new aws.Provider(
      "provider",
      { region: args.awsRegion as pulumi.Input<aws.Region> },
      { parent: this }
    );

    const connection = new aws.cloudwatch.EventConnection(
      "connection",
      {
        name: pulumi.interpolate`fn0-control-cron-${args.suffix}`,
        authorizationType: "API_KEY",
        authParameters: {
          apiKey: {
            key: "Authorization",
            value: pulumi.interpolate`Bearer ${args.controlAdminToken}`,
          },
        },
      },
      { parent: this, provider }
    );

    const apiDestination = new aws.cloudwatch.EventApiDestination(
      "api-destination",
      {
        name: pulumi.interpolate`fn0-control-cron-${args.suffix}`,
        connectionArn: connection.arn,
        httpMethod: "POST",
        invocationEndpoint: pulumi.interpolate`${args.controlUrl}/__forte_action/cron_on_tick`,
      },
      { parent: this, provider }
    );

    const role = new aws.iam.Role(
      "scheduler-role",
      {
        name: pulumi.interpolate`fn0-control-cron-scheduler-${args.suffix}`,
        assumeRolePolicy: JSON.stringify({
          Version: "2012-10-17",
          Statement: [
            {
              Effect: "Allow",
              Principal: { Service: "scheduler.amazonaws.com" },
              Action: "sts:AssumeRole",
            },
          ],
        }),
      },
      { parent: this, provider }
    );

    new aws.iam.RolePolicy(
      "scheduler-role-policy",
      {
        role: role.id,
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
      { parent: this, provider }
    );

    const schedule = new aws.scheduler.Schedule(
      "schedule",
      {
        name: pulumi.interpolate`fn0-control-cron-${args.suffix}`,
        scheduleExpression: "rate(1 minute)",
        flexibleTimeWindow: { mode: "OFF" },
        target: {
          arn: apiDestination.arn,
          roleArn: role.arn,
          input: '{"scheduled_time":"<aws.scheduler.scheduled-time>"}',
          retryPolicy: {
            maximumRetryAttempts: 0,
          },
        },
      },
      { parent: this, provider }
    );

    this.scheduleArn = schedule.arn;
    this.apiDestinationArn = apiDestination.arn;
    this.connectionArn = connection.arn;

    this.registerOutputs({
      scheduleArn: this.scheduleArn,
      apiDestinationArn: this.apiDestinationArn,
      connectionArn: this.connectionArn,
    });
  }
}
