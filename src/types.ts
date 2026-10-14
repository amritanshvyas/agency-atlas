export type FileStatus = "present" | "missing" | "ambiguous" | "not_available";

export interface Agent {
  id: string;
  name: string;
  description: string;
  category: string;
  categoryLabel: string;
  sourcePath: string;
  color?: string | null;
  emoji?: string | null;
  vibe?: string | null;
  metadataWarning?: string | null;
  mapping: {
    slug?: string | null;
    conflictingSources: string[];
    generated: FileStatus;
    installed: FileStatus;
  };
}

export interface Catalog {
  agents: Agent[];
  sourceRoot: string;
  installedDirectory?: string | null;
  generatedDirectory: string;
  warnings: string[];
}
