import { useState, useRef } from "react";
import Markdown from "react-markdown";
import {
  Heading,
  Bold,
  Italic,
  Quote,
  Code,
  Link,
  List,
  ListOrdered,
  CheckSquare,
} from "lucide-react";

type Mode = "write" | "preview";

export function MarkdownEditor({
  value,
  onChange,
  minHeight,
}: {
  value: string;
  onChange: (value: string) => void;
  minHeight: number;
}) {
  const [mode, setMode] = useState<Mode>("write");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const insertMarkdown = ({
    before,
    after,
  }: {
    before: string;
    after?: string;
  }) => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = value.substring(start, end);
    const afterText = after ?? before;

    const newText =
      value.substring(0, start) +
      before +
      selectedText +
      afterText +
      value.substring(end);

    onChange(newText);

    setTimeout(() => {
      textarea.focus();
      const newCursorPos = start + before.length + selectedText.length;
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  const insertHeading = () => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const lineStart = value.lastIndexOf("\n", start - 1) + 1;
    const lineEnd = value.indexOf("\n", start);
    const line = value.substring(
      lineStart,
      lineEnd === -1 ? value.length : lineEnd
    );

    const headingMatch = line.match(/^(#{1,6})\s/);
    let newLine: string;

    if (headingMatch && headingMatch[1]) {
      const currentLevel = headingMatch[1].length;
      if (currentLevel < 6) {
        newLine = line.replace(/^#{1,6}\s/, "#".repeat(currentLevel + 1) + " ");
      } else {
        newLine = line.replace(/^#{1,6}\s/, "");
      }
    } else {
      newLine = "# " + line;
    }

    const newText =
      value.substring(0, lineStart) +
      newLine +
      value.substring(lineEnd === -1 ? value.length : lineEnd);

    onChange(newText);

    setTimeout(() => {
      textarea.focus();
    }, 0);
  };

  const toolbarButtons = [
    {
      icon: Heading,
      title: "Heading",
      onClick: insertHeading,
    },
    {
      icon: Bold,
      title: "Bold",
      onClick: () => insertMarkdown({ before: "**" }),
    },
    {
      icon: Italic,
      title: "Italic",
      onClick: () => insertMarkdown({ before: "*" }),
    },
    {
      icon: Quote,
      title: "Quote",
      onClick: () => insertMarkdown({ before: "\n> ", after: "\n" }),
    },
    {
      icon: Code,
      title: "Code",
      onClick: () => insertMarkdown({ before: "`" }),
    },
    {
      icon: Link,
      title: "Link",
      onClick: () => insertMarkdown({ before: "[", after: "](url)" }),
    },
    {
      icon: List,
      title: "Unordered list",
      onClick: () => insertMarkdown({ before: "\n- ", after: "\n" }),
    },
    {
      icon: ListOrdered,
      title: "Ordered list",
      onClick: () => insertMarkdown({ before: "\n1. ", after: "\n" }),
    },
    {
      icon: CheckSquare,
      title: "Task list",
      onClick: () => insertMarkdown({ before: "\n- [ ] ", after: "\n" }),
    },
  ];

  return (
    <div className="border border-border rounded-md overflow-hidden">
      <div className="bg-muted/30">
        <div className="flex items-center gap-2 px-2 py-1 border-b border-border">
          <button
            type="button"
            onClick={() => setMode("write")}
            className={`px-3 py-1 text-sm rounded-md cursor-pointer ${
              mode === "write"
                ? "bg-background text-foreground"
                : "text-muted-foreground"
            }`}
          >
            書く
          </button>
          <button
            type="button"
            onClick={() => setMode("preview")}
            className={`px-3 py-1 text-sm rounded-md cursor-pointer ${
              mode === "preview"
                ? "bg-background text-foreground"
                : "text-muted-foreground"
            }`}
          >
            プレビュー
          </button>
        </div>

        {mode === "write" && (
          <div className="flex items-center gap-1 px-2 py-1 flex-wrap border-b border-border">
            {toolbarButtons.map(({ icon: Icon, title, onClick }, index) => (
              <button
                key={index}
                type="button"
                onClick={onClick}
                title={title}
                className="p-1.5 rounded cursor-pointer text-muted-foreground"
              >
                <Icon size={16} />
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="bg-background">
        {mode === "write" ? (
          <textarea
            ref={textareaRef}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            className={`w-full p-3 resize-y focus:outline-none bg-transparent min-h-[${minHeight}px]`}
          />
        ) : (
          <div className={`p-3 min-h-[${minHeight}px]`}>
            <div className="prose prose-sm max-w-none">
              <Markdown>{value}</Markdown>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
