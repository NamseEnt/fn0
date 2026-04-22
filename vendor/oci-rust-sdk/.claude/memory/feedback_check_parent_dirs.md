---
name: Check parent directories for config files
description: 워크플로우, CI 설정 등을 찾을 때 현재 디렉토리뿐 아니라 상위 디렉토리(.github 등)도 반드시 확인
type: feedback
---

프로젝트가 monorepo 구조일 수 있으므로, .github/workflows 등 설정 파일을 찾을 때 현재 작업 디렉토리뿐 아니라 상위 디렉토리도 확인해야 합니다.

**Why:** oci-rust-sdk는 fn0 monorepo의 하위 디렉토리이며, GitHub Actions 워크플로우는 `../.github/workflows/`에 위치함. 현재 디렉토리만 확인하고 "없다"고 답변하여 사용자에게 잘못된 정보를 전달함.

**How to apply:** CI/CD, 워크플로우, 배포 설정 등을 찾을 때 항상 git root 기준으로도 탐색할 것.
