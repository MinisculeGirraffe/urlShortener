import * as cdk from 'aws-cdk-lib';
import { Architecture, FunctionUrlAuthType } from 'aws-cdk-lib/aws-lambda';
import { RustFunction } from 'cargo-lambda-cdk';
import { Construct } from 'constructs';
import path = require('path');

export class UrlShortenerStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);


    const table = new cdk.aws_dynamodb.Table(this, "UrlShortenerTable", {
      partitionKey: { name: "slug", type: cdk.aws_dynamodb.AttributeType.STRING },
      billingMode: cdk.aws_dynamodb.BillingMode.PAY_PER_REQUEST,
      timeToLiveAttribute: "ttl"
    })


    const handler_lambda = new RustFunction(this, "url_shortener_lambda", {
      manifestPath: path.join(__dirname, '..', '..', "src", "url_shortener_lambda"),
      bundling: { architecture: Architecture.ARM_64 },
      environment: { table_name: table.tableName },
      architecture: Architecture.ARM_64,
      logRetention: cdk.aws_logs.RetentionDays.ONE_DAY,
      memorySize: 128,
    })

    table.grantReadWriteData(handler_lambda)

    let url = new cdk.aws_lambda.FunctionUrl(this, "handlerUrl", {
      authType: FunctionUrlAuthType.NONE,
      function: handler_lambda
    })

    new cdk.CfnOutput(this, "Function Url", { value: url.url })
  }

}
