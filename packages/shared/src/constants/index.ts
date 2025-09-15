export const DEFAULT_TREE_PARAMETERS = {
  seed: 123,
  height: 5.0,
  thickness: 1.0,
  branchiness: 0.7,
  leafDensity: 0.5,
};

export const TREE_SPECIES = {
  OAK: 'oak',
  PINE: 'pine', 
  BIRCH: 'birch',
  MAPLE: 'maple',
} as const;

export const SCREEN_ROUTES = {
  TREE_DESIGNER: '/tree-designer',
  FOREST_VIEWER: '/forest-viewer',
} as const;

export const TOOL_CATEGORIES = {
  GENERATION: 'generation',
  MODIFICATION: 'modification', 
  VISUALIZATION: 'visualization',
  EXPORT: 'export',
} as const;