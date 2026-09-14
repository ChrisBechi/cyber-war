import data from '../../content/software/kali-default.json';

export type SoftwareEntry = (typeof data.entries)[number];
export type SoftwareCategory = (typeof data.categories)[number];
export const softwareCatalog = data;
export const softwareById = new Map(data.entries.map((entry) => [entry.id, entry]));
export const categoryById = new Map(data.categories.map((category) => [category.id, category]));

export function inCategory(entry: SoftwareEntry, category: string): boolean {
  return entry.categories.some((id) => {
    let current: string | null = id;
    while (current) {
      if (current === category) {
        return true;
      }
      current = categoryById.get(current)?.parent ?? null;
    }
    return false;
  });
}

export function searchSoftware(query: string): SoftwareEntry[] {
  const words = query.trim().toLocaleLowerCase().split(/\s+/);
  return data.entries
    .filter((entry) => {
      const searchable =
        `${entry.name} ${entry.package} ${entry.description} ${entry.commands.join(' ')}`.toLocaleLowerCase();
      return words.every((word) => searchable.includes(word));
    })
    .sort(
      (a, b) =>
        Number(b.name.toLowerCase() === query.toLowerCase()) -
          Number(a.name.toLowerCase() === query.toLowerCase()) || a.name.localeCompare(b.name),
    );
}

export function launcherList(value: string | undefined, fallback: string[] = []): string[] {
  try {
    const parsed: unknown = value ? JSON.parse(value) : fallback;
    return Array.isArray(parsed)
      ? [
          ...new Set(
            parsed.filter((id): id is string => typeof id === 'string' && softwareById.has(id)),
          ),
        ]
      : fallback;
  } catch {
    return fallback;
  }
}
