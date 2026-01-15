import { Link } from "@/components/ui/link";

export function NewsItem({
  item,
}: {
  item: {
    id: string;
    title: string;
    url: string;
    domain: string;
  };
}) {
  return (
    <div className="py-3 border-b">
      <div className="flex items-baseline gap-2 flex-wrap">
        <Link href={`/post/${item.id}`} size="base" weight="medium">
          {item.title}
        </Link>
        <Link href={item.url} size="sm" muted popup>
          {item.domain}
        </Link>
      </div>
    </div>
  );
}
