cli deploy에 대응하는 api를 만들어야함.

1. 유저 auth를 할 수 있어야함.
2. 빌드할 것들을 올리는 s3의 presigned url 제공
3. 전처리를 위해 aws lambda invoke를 진행
4. 람다가 site 코드 저장소에 쏜다. 나중엔 cloudflare workers가 쏠 수도 있다.

데이터베이스는 doc db 쓰고

cli: [나 누구누구고, 코드 올릴 준비됨]
hq: [인증 완료. 여기에 올려]
cli: [다 올림. 사이트에 배포해주셈]
hq: [전처리 완료하고 사이트들에 배포함]

그렇다면 위처럼 api는 2개만 있으면 될듯

deploy_01_start
deploy_02_finish

이렇게 만들면 되겠다.
