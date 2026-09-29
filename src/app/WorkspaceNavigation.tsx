import { Activity, Code2, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { DiagnosticSectionId } from "@/features/diagnostics/catalog";

const WORKSPACES = [
  { id: "overview", label: "Diagnostika", icon: Activity },
  { id: "coding", label: "Kódování", icon: Code2 },
  { id: "service", label: "Servis", icon: Wrench },
] as const;

export function WorkspaceNavigation({
  selected,
  onSelect,
}: {
  selected: DiagnosticSectionId;
  onSelect: (section: DiagnosticSectionId) => void;
}) {
  const active =
    selected === "coding" || selected === "service" ? selected : "overview";
  return (
    <nav
      aria-label="Hlavní části aplikace"
      className="flex shrink-0 items-center gap-2 border-b bg-card/50 px-3 py-2"
    >
      {WORKSPACES.map(({ id, label, icon: Icon }) => (
        <Button
          key={id}
          type="button"
          variant={active === id ? "secondary" : "ghost"}
          aria-current={active === id ? "page" : undefined}
          onClick={() => onSelect(id)}
        >
          <Icon aria-hidden="true" />
          {label}
        </Button>
      ))}
      <span className="ml-auto hidden text-xs text-muted-foreground md:inline">
        Katalog úprav lze procházet i bez vozidla
      </span>
    </nav>
  );
}
