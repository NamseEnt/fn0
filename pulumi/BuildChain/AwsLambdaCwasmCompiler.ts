import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";

export interface AwsLambdaCwasmCompilerArgs {
  region: pulumi.Input<string>;
  bucket: pulumi.Input<string>;
  queueArn: pulumi.Input<string>;
}

export class AwsLambdaCwasmCompiler extends pulumi.ComponentResource {
  constructor(
    name: string,
    args: AwsLambdaCwasmCompilerArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:aws-lambda-cwasm-compiler", name, args, opts);

    const { region, bucket, queueArn } = args;

    const bucketArn = pulumi.interpolate`arn:aws:s3:::${bucket}`;

    const lambda = new aws.lambda.Function("lambda", {
      role: new aws.iam.Role("role", {
        assumeRolePolicy: {
          Version: "2012-10-17",
          Statement: [
            {
              Effect: "Allow",
              Principal: {
                Service: "lambda.amazonaws.com",
              },
              Action: "sts:AssumeRole",
            },
            {
              Effect: "Allow",
              Principal: {
                Service: "sqs.amazonaws.com",
              },
              Action: "sts:AssumeRole",
            },
          ],
        },
        inlinePolicies: [
          {
            name: "watchdog-policy",
            policy: pulumi
              .output({
                Version: "2012-10-17",
                Statement: [
                  {
                    Effect: "Allow",
                    Action: ["s3:GetObject", "s3:DeleteObject"],
                    Resource: bucketArn.apply((arn) => `${arn}/*`),
                  },
                  {
                    Effect: "Allow",
                    Action: ["s3:ListBucket"],
                    Resource: bucketArn,
                  },
                ],
              } satisfies aws.iam.PolicyDocument)
              .apply((policyDoc) => JSON.stringify(policyDoc)),
          },
        ],
        managedPolicyArns: [
          aws.iam.ManagedPolicy.AWSLambdaBasicExecutionRole,
          aws.iam.ManagedPolicy.AWSLambdaSQSQueueExecutionRole,
        ],
      }).arn,
    });
    // 3. IAM 권한 부여 (중요)
    // CallbackFunction이 기본 Role을 만들지만, SQS를 '읽을' 권한은 명시적으로 주어야 합니다.
    // AWS에서 제공하는 'AWSLambdaSQSQueueExecutionRole' 관리형 정책을 붙입니다.
    const lambdaSqsPolicy = new aws.iam.RolePolicyAttachment(
      "my-lambda-sqs-policy",
      {
        role: myLambda.role, // 위에서 생성된 Lambda의 IAM Role
        policyArn:
          "arn:aws:iam::aws:policy/service-role/AWSLambdaSQSQueueExecutionRole",
      }
    );

    // 4. Event Source Mapping (트리거 설정)
    // 이 리소스가 실제로 SQS와 Lambda를 연결해줍니다.
    const eventSourceMapping = new aws.lambda.EventSourceMapping(
      "my-trigger",
      {
        eventSourceArn: queue.arn,
        functionName: myLambda.name,
        batchSize: 10, // 한 번에 Lambda로 전달할 메시지 개수 (최대 10,000)
      },
      { dependsOn: [lambdaSqsPolicy] }
    ); // 권한 설정이 완료된 후 연결해야 안전합니다.
  }
}
