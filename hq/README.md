hq is headquarter

this does everything for fn0 as control panel

1. wasm hosts health check
2. wasm hosts user code version metadata sync
3. user code deployment

\

1. host infra는 10초마다 호스트 인스턴스 정보를 가져와서 host info map 이라는
   dashmap를 업데이트해라. 맵에 있으나 list api에는 사라진 경우는 맵에서 지우라는 뜻.\
2. health checker는 5초마다 host info map을 읽어서 호스트별로 task를 만들어, 0~1초
   사이의 random sleep 후 host에 /health 요청, 타임아웃 5초로 하여, 정상 응답일때만
   dashmap health map에 업데이트해라.\
3. reaper는 10초마다, health check된지 15초가 넘었고, host info map에 나타난진
   60초가 넘었거나 Terminating 상태가 아니고, 지난 틱에서 terminate 명령을 내리지 않은
   인스턴스에 대해 host_infra의 도움을 받아 해당 instance를 죽이는 task를 spawn하고
   랜덤 슬립 0~1초 넣고 죽여라.\
4. dns-syncer는 5초마다, healthy(=health check한지 7.5초 이하)인 인스턴스들을 dns와
   싱크를 맞춰야한다. 단, 매번 api 호출할 필요 없이 인메모리 캐시와 비교하여 변경사항
   없으면 api를 보내지 않는다.\
