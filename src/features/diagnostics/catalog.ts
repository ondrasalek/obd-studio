import type { LucideIcon } from "lucide-react";
import {
  Activity,
  Armchair,
  BatteryCharging,
  CarFront,
  Code2,
  Disc3,
  DoorOpen,
  Fan,
  Filter,
  Fuel,
  Gauge,
  LayoutDashboard,
  Lightbulb,
  PanelsTopLeft,
  Radio,
  Settings2,
  SlidersHorizontal,
  Snowflake,
  Wrench,
} from "lucide-react";
import type { ControlUnitArea, LiveDataGroup } from "@/types/diagnostics";

export type DiagnosticSectionId =
  | "overview"
  | "diagnostics"
  | "engine"
  | "injectors"
  | "dpf"
  | "turbo"
  | "electrical"
  | "emissions"
  | "fuel"
  | "transmission"
  | "drivingModes"
  | "brakes"
  | "interior"
  | "body"
  | "windows"
  | "lighting"
  | "exterior"
  | "climate"
  | "infotainment"
  | "performance"
  | "coding"
  | "service";

export interface DiagnosticSection {
  id: DiagnosticSectionId;
  title: string;
  description: string;
  icon: LucideIcon;
  liveGroup?: LiveDataGroup;
  controlUnitArea?: ControlUnitArea;
  availability: "core" | "profiled" | "planned" | "utility";
  modules?: string[];
}

export interface DiagnosticSectionGroup {
  label: string;
  sections: DiagnosticSection[];
}

export const DIAGNOSTIC_SECTION_GROUPS: DiagnosticSectionGroup[] = [
  {
    label: "Vozidlo",
    sections: [
      {
        id: "overview",
        title: "Přehled vozidla",
        description: "Stav spojení a rychlý přístup",
        icon: LayoutDashboard,
        availability: "core",
      },
      {
        id: "diagnostics",
        title: "Čtení a měření",
        description: "Přehled dostupných diagnostických funkcí",
        icon: Activity,
        availability: "core",
      },
      {
        id: "electrical",
        title: "Elektrická síť",
        description: "Napětí ECU a orientační stav dobíjení",
        icon: BatteryCharging,
        liveGroup: "electrical",
        availability: "core",
      },
    ],
  },
  {
    label: "Hnací ústrojí",
    sections: [
      {
        id: "engine",
        title: "Motor",
        description: "Provozní hodnoty motorové ECU",
        icon: Gauge,
        liveGroup: "engine",
        availability: "core",
      },
      {
        id: "injectors",
        title: "Vstřikování",
        description: "Palivový tlak, směs a profilové korekce válců",
        icon: Fuel,
        liveGroup: "injectors",
        availability: "core",
      },
      {
        id: "dpf",
        title: "DPF",
        description: "Teploty a tlak; profilově saze, popel a regenerace",
        icon: Filter,
        liveGroup: "dpf",
        availability: "core",
      },
      {
        id: "turbo",
        title: "Turbo",
        description: "Plnicí tlak a teploty",
        icon: Fan,
        liveGroup: "turbo",
        availability: "core",
      },
      {
        id: "fuel",
        title: "Palivová soustava",
        description: "Hladina, tlak v liště a průtok paliva",
        icon: Fuel,
        liveGroup: "fuel",
        availability: "core",
      },
      {
        id: "emissions",
        title: "Emise a směs",
        description: "Korekce směsi, lambda a NOx",
        icon: Activity,
        liveGroup: "emissions",
        availability: "core",
      },
      {
        id: "transmission",
        title: "Převodovka",
        description: "DSG a automatické převodovky",
        icon: Settings2,
        controlUnitArea: "transmission",
        availability: "profiled",
        modules: [
          "Řídicí jednotka převodovky",
          "Pohon všech kol",
          "Volicí páka převodovky",
        ],
      },
      {
        id: "drivingModes",
        title: "Jízdní režimy",
        description: "ECU zapojené do volby jízdního profilu",
        icon: SlidersHorizontal,
        controlUnitArea: "drivingModes",
        availability: "profiled",
        modules: [
          "Motor a převodovka",
          "Řízení a adaptivní podvozek",
          "Pohon všech kol",
          "Gateway a infotainment",
        ],
      },
    ],
  },
  {
    label: "Podvozek",
    sections: [
      {
        id: "brakes",
        title: "Brzdy a podvozek",
        description: "ABS, ESC a asistenční systémy",
        icon: Disc3,
        controlUnitArea: "chassis",
        availability: "profiled",
        modules: [
          "ABS / ESC",
          "Elektronická parkovací brzda",
          "Posilovač řízení",
          "Adaptivní podvozek DCC",
          "Adaptivní tempomat",
          "Kamera asistenčních systémů",
        ],
      },
    ],
  },
  {
    label: "Karoserie a komfort",
    sections: [
      {
        id: "interior",
        title: "Interiér",
        description: "Přístroje, bezpečnost a sedadla",
        icon: Armchair,
        controlUnitArea: "interior",
        availability: "profiled",
        modules: [
          "Přístrojový panel",
          "Airbag a sloupek řízení",
          "Sedadlo řidiče a spolujezdce",
        ],
      },
      {
        id: "body",
        title: "Karoserie",
        description: "Centrální komfort a přístupové systémy",
        icon: PanelsTopLeft,
        controlUnitArea: "body",
        availability: "profiled",
        modules: [
          "Centrální elektrika J519",
          "Komfortní systém a KESSY",
          "Gateway",
          "Elektrické víko a střecha",
        ],
      },
      {
        id: "windows",
        title: "Okna a dveře",
        description: "Dveřní jednotky a ovládání oken",
        icon: DoorOpen,
        controlUnitArea: "windows",
        availability: "profiled",
        modules: [
          "Okno a dveře řidiče",
          "Okno a dveře spolujezdce",
          "Levé a pravé zadní dveře",
        ],
      },
      {
        id: "lighting",
        title: "Světla",
        description: "J519 a světelné funkce",
        icon: Lightbulb,
        controlUnitArea: "lighting",
        availability: "profiled",
        modules: [
          "Centrální elektrika J519",
          "Regulace sklonu světlometů",
          "Asistent dálkových světel",
        ],
      },
      {
        id: "exterior",
        title: "Exteriér",
        description: "Parkování, kamery a víka",
        icon: CarFront,
        controlUnitArea: "exterior",
        availability: "profiled",
        modules: [
          "Parkovací asistence",
          "Zadní kamera",
          "Elektrické víko",
          "Tažné zařízení",
          "Asistent změny jízdního pruhu",
        ],
      },
      {
        id: "climate",
        title: "Klimatizace",
        description: "Climatronic a teplotní zóny",
        icon: Snowflake,
        controlUnitArea: "climate",
        availability: "profiled",
        modules: [
          "Climatronic",
          "Přídavné topení",
          "Zadní ovládání klimatizace",
          "Kompresor klimatizace",
        ],
      },
      {
        id: "infotainment",
        title: "Infotainment",
        description: "MIB, rádio, navigace a zvuk",
        icon: Radio,
        controlUnitArea: "infotainment",
        availability: "profiled",
        modules: [
          "Informační elektronika MIB",
          "Rádio a navigace",
          "Zvukový systém",
          "Telematika a telefonní rozhraní",
        ],
      },
    ],
  },
  {
    label: "Pokročilé funkce",
    sections: [
      {
        id: "performance",
        title: "Výkonové testy",
        description: "0–100 km/h, 1/4 míle a grafy",
        icon: Gauge,
        availability: "planned",
      },
      {
        id: "coding",
        title: "Kódování",
        description: "Komfort, světla, okna a přístroje",
        icon: Code2,
        availability: "utility",
      },
      {
        id: "service",
        title: "Servisní funkce",
        description: "DTC, intervaly, brzdy a adaptace",
        icon: Wrench,
        availability: "utility",
      },
    ],
  },
];

export const DIAGNOSTIC_SECTIONS = DIAGNOSTIC_SECTION_GROUPS.flatMap(
  (group) => group.sections,
);

export function getDiagnosticSection(id: DiagnosticSectionId) {
  return (
    DIAGNOSTIC_SECTIONS.find((section) => section.id === id) ??
    DIAGNOSTIC_SECTIONS[0]
  );
}
