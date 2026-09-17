import { CheckCircle2, ChevronRight, Clock3, Radio } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type {
  ControlUnitArea,
  ControlUnitAreaSnapshot,
  DiagnosticProfile,
  LiveDataGroup,
  LiveDataSnapshot,
} from "@/types/diagnostics";
import {
  DIAGNOSTIC_SECTIONS,
  DIAGNOSTIC_SECTION_GROUPS,
  type DiagnosticSection,
  type DiagnosticSectionId,
} from "./catalog";

interface DiagnosticNavigationProps {
  selected: DiagnosticSectionId;
  profile: DiagnosticProfile;
  connected: boolean;
  snapshots: Partial<Record<LiveDataGroup, LiveDataSnapshot>>;
  controlUnitSnapshots: Partial<
    Record<ControlUnitArea, ControlUnitAreaSnapshot>
  >;
  onSelect: (section: DiagnosticSectionId) => void;
}

export function DiagnosticNavigation({
  selected,
  profile,
  connected,
  snapshots,
  controlUnitSnapshots,
  onSelect,
}: DiagnosticNavigationProps) {
  return (
    <Card className="min-h-0 flex-1 overflow-hidden">
      <CardHeader className="border-b pb-3">
        <div className="flex items-center justify-between gap-3">
          <div>
            <CardTitle>Diagnostické oblasti</CardTitle>
            <CardDescription>Vyberte systém vozidla</CardDescription>
          </div>
          <Badge variant="outline" className="font-mono text-[10px]">
            {DIAGNOSTIC_SECTIONS.length} oblastí
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="min-h-0 flex-1 px-2 py-2">
        <ScrollArea className="h-full">
          <nav aria-label="Diagnostické oblasti" className="space-y-4 py-1">
            {DIAGNOSTIC_SECTION_GROUPS.map((group) => (
              <div key={group.label}>
                <p className="mb-1.5 px-2 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground/70">
                  {group.label}
                </p>
                <div className="space-y-0.5">
                  {group.sections.map((section) => (
                    <NavigationItem
                      key={section.id}
                      section={section}
                      selected={selected === section.id}
                      profiled={profile === "vwMqb"}
                      connected={connected}
                      loaded={Boolean(
                        (section.liveGroup && snapshots[section.liveGroup]) ||
                          (section.controlUnitArea &&
                            controlUnitSnapshots[section.controlUnitArea]),
                      )}
                      onSelect={onSelect}
                    />
                  ))}
                </div>
              </div>
            ))}
          </nav>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

function NavigationItem({
  section,
  selected,
  profiled,
  connected,
  loaded,
  onSelect,
}: {
  section: DiagnosticSection;
  selected: boolean;
  profiled: boolean;
  connected: boolean;
  loaded: boolean;
  onSelect: (section: DiagnosticSectionId) => void;
}) {
  const Icon = section.icon;
  const profiledReady = section.availability === "profiled" && profiled;

  return (
    <Button
      type="button"
      variant="ghost"
      className={cn(
        "group h-auto w-full justify-start gap-2.5 px-2.5 py-2 text-left",
        selected && "bg-accent text-accent-foreground",
      )}
      onClick={() => onSelect(section.id)}
      aria-current={selected ? "page" : undefined}
    >
      <span
        className={cn(
          "flex size-8 shrink-0 items-center justify-center rounded-md border bg-background/60 text-muted-foreground",
          selected && "border-primary/30 bg-primary/10 text-primary",
        )}
      >
        <Icon className="size-4" aria-hidden="true" />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-xs font-medium">{section.title}</span>
        <span className="mt-0.5 block truncate text-[10px] font-normal text-muted-foreground">
          {section.description}
        </span>
      </span>
      {loaded ? (
        <CheckCircle2 className="size-3.5 shrink-0 text-emerald-400" aria-label="Data načtena" />
      ) : profiledReady && connected ? (
        <Radio className="size-3.5 shrink-0 text-cyan-400" aria-label="Připraveno ke čtení" />
      ) : section.availability === "planned" ? (
        <Clock3 className="size-3.5 shrink-0 text-muted-foreground/60" aria-label="Připravujeme" />
      ) : (
        <ChevronRight className="size-3.5 shrink-0 text-muted-foreground/50 transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
      )}
    </Button>
  );
}
