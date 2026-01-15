import { cn } from "@/lib/utils";

export function IconButton({
  children,
  className,
  ...props
}: {
  children: React.ReactNode;
  className?: string;
} & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      className={cn("p-2 rounded-full cursor-pointer", className)}
      {...props}
    >
      {children}
    </button>
  );
}
