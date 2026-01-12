import { NewsHeader } from "./NewsHeader";
import { Footer } from "./Footer";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <>
      <NewsHeader />
      <main className="container mx-auto px-4 py-6">{children}</main>
      <Footer />
    </>
  );
}
