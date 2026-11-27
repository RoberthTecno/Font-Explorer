export interface Tag {
  id: number;
  name: string;
  color: string | null;
}

export interface UserData {
  favorites: string[];
  tags: Tag[];
  fontTags: { fontId: string; tagId: number }[];
  /** Ids de fuentes abiertas, la más reciente primero. */
  recents: string[];
  settings: Record<string, string>;
}
