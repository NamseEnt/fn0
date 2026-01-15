# Claude Code Guidelines for ls-news

## Page Development Flow

페이지 개발 순서:
1. `rs/src/pages/` 에서 props를 return하는 handler 구현
2. `../forte/forte-rs-to-fe` 에서 `cargo run` 실행하여 fe쪽에 `.props.ts` 자동생성
3. `fe/src/pages/.../page.tsx` 에서 `export default function PageName(props: Props) {...}` 구현

## Frontend (React/TypeScript)

### Component Props

- **Page components**: Props는 `.props.ts` 파일에서 import하여 사용
  ```tsx
  import type { Props } from "./.props";
  export default function IndexPage(props: Props) { ... }
  ```

- **일반 컴포넌트**: Props는 항상 inline으로 정의
  ```tsx
  // Good
  export function NewsItem({
    item,
  }: {
    item: { id: string; title: string; };
  }) { ... }

  // Bad - 별도 type 정의 금지
  type NewsItemProps = { item: { id: string; title: string; } };
  export function NewsItem({ item }: NewsItemProps) { ... }
  ```

## Environment Variables

- `TURSO_URL`: libsql 데이터베이스 URL (예: `http://localhost:8080`)
- `TURSO_AUTH_TOKEN`: 인증 토큰 (로컬 개발시 빈 문자열)
