# Claude Code Guidelines for ls-news

## Page Development Flow

Page development order:
1. Implement a handler that returns props in `rs/src/pages/`
2. Run `cargo run` in `../forte/forte-rs-to-fe` to auto-generate `.props.ts` on the fe side
3. Implement `export default function PageName(props: Props) {...}` in `fe/src/pages/.../page.tsx`

## Frontend (React/TypeScript)

### Component Props

- **Page components**: Import and use Props from `.props.ts` files
  ```tsx
  import type { Props } from "./.props";
  export default function IndexPage(props: Props) { ... }
  ```

- **Regular components**: Props should always be defined inline
  ```tsx
  // Good
  export function NewsItem({
    item,
  }: {
    item: { id: string; title: string; };
  }) { ... }

  // Bad - separate type definition not allowed
  type NewsItemProps = { item: { id: string; title: string; } };
  export function NewsItem({ item }: NewsItemProps) { ... }
  ```

## Environment Variables

- `TURSO_URL`: libsql 데이터베이스 URL (예: `http://localhost:8080`)
- `TURSO_AUTH_TOKEN`: 인증 토큰 (로컬 개발시 빈 문자열)
