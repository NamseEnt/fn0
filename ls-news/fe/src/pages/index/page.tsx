import type { Props } from "./.props";
import { Layout } from "@/components/Layout";
import { NewsItem } from "@/components/NewsItem";
import { NewsPagination } from "@/components/NewsPagination";

export default function IndexPage(props: Props) {
  if (props.t === "DbErr") {
    return (
      <Layout>
        <div className="text-center text-red-500">
          Db Error: {props.message}
        </div>
      </Layout>
    );
  }

  const displayedItems = props.rows.map((row) => ({
    id: row.post.id,
    title: row.post.title,
    url: row.post.url,
    domain: getDomain(row.post.url),
    author: row.author,
    deletedAt: row.deletedAt,
  }));

  const hasMore = displayedItems.length === 10;

  return (
    <Layout>
      {displayedItems.map((item) => (
        <NewsItem key={item.id} item={item} />
      ))}
      {displayedItems.length > 0 && (
        <NewsPagination
          lastKey={displayedItems[displayedItems.length - 1].id}
          hasMore={hasMore}
        />
      )}
    </Layout>
  );
}

function getDomain(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}
