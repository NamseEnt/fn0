Need to build an API corresponding to cli deploy.

1. Must support user auth.
2. Provide presigned URL for S3 to upload build artifacts.
3. Invoke AWS Lambda for preprocessing.
4. Lambda pushes to the site code storage. Later, Cloudflare Workers may handle this.

Use Doc DB for the database.

cli: [I am so-and-so, ready to upload code]
hq: [Auth complete. Upload here]
cli: [Done uploading. Deploy to site please]
hq: [Preprocessing complete, deployed to sites]

So it looks like we only need 2 APIs:

deploy_01_start
deploy_02_finish

That should be the approach.
