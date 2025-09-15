import { TreeParameters, TreeSpecies } from '../types';
import { DEFAULT_TREE_PARAMETERS, TREE_SPECIES } from '../constants';

export function createDefaultTreeParameters(): TreeParameters {
  return { ...DEFAULT_TREE_PARAMETERS };
}

export function validateTreeParameters(params: Partial<TreeParameters>): TreeParameters {
  return {
    seed: params.seed ?? DEFAULT_TREE_PARAMETERS.seed,
    height: Math.max(0.1, params.height ?? DEFAULT_TREE_PARAMETERS.height),
    thickness: Math.max(0.1, params.thickness ?? DEFAULT_TREE_PARAMETERS.thickness),
    branchiness: Math.max(0, Math.min(1, params.branchiness ?? DEFAULT_TREE_PARAMETERS.branchiness)),
    leafDensity: Math.max(0, Math.min(1, params.leafDensity ?? DEFAULT_TREE_PARAMETERS.leafDensity)),
    bark_texture: params.bark_texture,
    leaf_texture: params.leaf_texture,
  };
}

export function getTreeSpeciesById(id: string): TreeSpecies | undefined {
  const speciesMap: Record<string, TreeSpecies> = {
    [TREE_SPECIES.OAK]: {
      id: TREE_SPECIES.OAK,
      name: 'Oak Tree',
      description: 'A sturdy deciduous tree with broad leaves and strong branches',
      parameters: {
        ...DEFAULT_TREE_PARAMETERS,
        branchiness: 0.8,
        leafDensity: 0.7,
      }
    },
    [TREE_SPECIES.PINE]: {
      id: TREE_SPECIES.PINE,
      name: 'Pine Tree', 
      description: 'A tall evergreen conifer with needle-like leaves',
      parameters: {
        ...DEFAULT_TREE_PARAMETERS,
        height: 8.0,
        branchiness: 0.4,
        leafDensity: 0.9,
      }
    },
    [TREE_SPECIES.BIRCH]: {
      id: TREE_SPECIES.BIRCH,
      name: 'Birch Tree',
      description: 'A slender deciduous tree with distinctive white bark',
      parameters: {
        ...DEFAULT_TREE_PARAMETERS,
        thickness: 0.7,
        branchiness: 0.6,
        leafDensity: 0.6,
      }
    },
    [TREE_SPECIES.MAPLE]: {
      id: TREE_SPECIES.MAPLE,
      name: 'Maple Tree',
      description: 'A medium-sized tree known for its colorful autumn foliage',
      parameters: {
        ...DEFAULT_TREE_PARAMETERS,
        branchiness: 0.7,
        leafDensity: 0.8,
      }
    }
  };
  
  return speciesMap[id];
}