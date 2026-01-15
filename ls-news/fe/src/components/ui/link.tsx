import { cn } from "@/lib/utils";

const fontSizes = {
  sm: "text-sm",
  base: "text-base",
};

const fontWeights = {
  thin: "font-thin",
  extralight: "font-extralight",
  light: "font-light",
  normal: "font-normal",
  medium: "font-medium",
  semibold: "font-semibold",
  bold: "font-bold",
  extrabold: "font-extrabold",
  black: "font-black",
} as const;

export function Link({
  href,
  popup,
  size,
  weight,
  muted,
  children,
  ...props
}: {
  href: string;
  popup?: boolean;
  size?: keyof typeof fontSizes;
  weight?: keyof typeof fontWeights;
  muted?: boolean;
  children?: React.ReactNode;
} & Omit<React.AnchorHTMLAttributes<HTMLAnchorElement>, "className" | "href">) {
  const popupProps = popup
    ? { target: "_blank", rel: "noopener noreferrer" }
    : {};

  return (
    <a
      href={href}
      className={cn(
        "hover:underline",
        size ? fontSizes[size] : "",
        weight ? fontWeights[weight] : "",
        muted ? "text-muted-foreground" : ""
      )}
      {...popupProps}
      {...props}
    >
      {children}
    </a>
  );
}
