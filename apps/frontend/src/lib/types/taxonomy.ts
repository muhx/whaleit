// Taxonomy-related type definitions

export interface Taxonomy {
  id: string;
  name: string;
  color: string;
  description?: string | null;
  isSystem: boolean;
  isSingleSelect: boolean;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface TaxonomyCategory {
  id: string;
  taxonomyId: string;
  parentId?: string | null;
  name: string;
  key: string;
  color: string;
  description?: string | null;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface TaxonomyWithCategories {
  taxonomy: Taxonomy;
  categories: TaxonomyCategory[];
}

export interface NewTaxonomy {
  id?: string | null;
  name: string;
  color: string;
  description?: string | null;
  isSystem: boolean;
  isSingleSelect: boolean;
  sortOrder: number;
}

export interface NewTaxonomyCategory {
  id?: string | null;
  taxonomyId: string;
  parentId?: string | null;
  name: string;
  key: string;
  color: string;
  description?: string | null;
  sortOrder: number;
}

export interface TaxonomyJson {
  name: string;
  color: string;
  categories: TaxonomyCategoryJson[];
  instruments?: TaxonomyInstrumentMappingJson[];
}

export interface TaxonomyCategoryJson {
  name: string;
  key: string;
  color: string;
  description?: string | null;
  children: TaxonomyCategoryJson[];
}

export interface TaxonomyInstrumentMappingJson {
  isin?: string | null;
  symbol?: string | null;
  categoryKey: string;
  weight: number;
}

export interface CategoryRef {
  id: string;
  name: string;
}

export interface CategoryWithWeight {
  category: TaxonomyCategory;
  // The top-level ancestor category (for hierarchical taxonomies like GICS)
  // Used for filtering when allocations are rolled up to top-level
  topLevelCategory: CategoryRef;
  weight: number; // 0-100 percentage
}

export interface CategoryAllocation {
  categoryId: string;
  categoryName: string;
  color: string;
  value: number; // Base currency value
  percentage: number; // 0-100
  children?: CategoryAllocation[]; // Child allocations for drill-down
}

export interface TaxonomyAllocation {
  taxonomyId: string;
  taxonomyName: string;
  color: string;
  categories: CategoryAllocation[];
}

export interface MigrationStatus {
  needed: boolean;
  assetsWithLegacyData: number;
  assetsAlreadyMigrated: number;
}

export interface MigrationResult {
  sectorsMigrated: number;
  countriesMigrated: number;
  assetsProcessed: number;
  errors: string[];
}
