import type { Agent } from "./types";

export function filterAgents(agents: Agent[], query: string, category: string): Agent[] {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return agents.filter((agent) => {
    if (category !== "all" && agent.category !== category) return false;
    const haystack = [agent.name, agent.description, agent.category, agent.categoryLabel, agent.sourcePath]
      .join(" ").toLocaleLowerCase();
    return terms.every((term) => haystack.includes(term));
  });
}

export function categoryCounts(agents: Agent[]): Array<{ id: string; label: string; count: number }> {
  const categories = new Map<string, { label: string; count: number }>();
  for (const agent of agents) {
    const item = categories.get(agent.category);
    if (item) item.count++;
    else categories.set(agent.category, { label: agent.categoryLabel, count: 1 });
  }
  return [...categories].map(([id, value]) => ({ id, ...value }))
    .sort((a, b) => a.label.localeCompare(b.label));
}
