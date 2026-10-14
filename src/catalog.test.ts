import { describe, expect, it } from "vitest";
import { categoryCounts, filterAgents } from "./catalog";
import type { Agent } from "./types";

const makeAgent = (id: string, name: string, description: string, category: string): Agent => ({
  id, name, description, category, categoryLabel: category === "engineering" ? "Engineering" : "Design",
  sourcePath: id, mapping: { slug: null, conflictingSources: [], generated: "not_available", installed: "not_available" }
});

const agents = [
  makeAgent("engineering/nested/api-expert.md", "API Expert", "Builds reliable APIs", "engineering"),
  makeAgent("design/visual.md", "Visual Designer", "Design systems", "design"),
  makeAgent("engineering/ui.md", "UI Engineer", "Builds interfaces", "engineering")
];

describe("catalog filtering", () => {
  it("searches name, description, category and source path", () => {
    expect(filterAgents(agents, "api expert", "all").map((a) => a.id)).toEqual([agents[0].id]);
    expect(filterAgents(agents, "reliable", "all").map((a) => a.id)).toEqual([agents[0].id]);
    expect(filterAgents(agents, "engineering", "all")).toHaveLength(2);
    expect(filterAgents(agents, "nested", "all").map((a) => a.id)).toEqual([agents[0].id]);
  });

  it("combines search with category filter and returns empty results", () => {
    expect(filterAgents(agents, "engineer", "design")).toEqual([]);
    expect(filterAgents(agents, "   ", "design").map((a) => a.id)).toEqual([agents[1].id]);
  });

  it("counts categories without conflating IDs", () => {
    expect(categoryCounts(agents)).toEqual([
      { id: "design", label: "Design", count: 1 },
      { id: "engineering", label: "Engineering", count: 2 }
    ]);
  });
});
