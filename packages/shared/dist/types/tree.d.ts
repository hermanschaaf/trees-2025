export interface TreeRing {
    center: [number, number, number];
    radius: number;
    orientation: [number, number, number, number];
    growth_factor: number;
    bark_thickness: number;
}
export interface TreeStructure {
    rings: TreeRing[];
    parent_indices: (number | null)[];
}
export interface TreeSpecies {
    id: string;
    name: string;
    description: string;
    parameters: TreeParameters;
    previewImage?: string;
}
export interface TreeParameters {
    seed: number;
    height: number;
    thickness: number;
    branchiness: number;
    leafDensity: number;
    bark_texture?: string;
    leaf_texture?: string;
}
export interface TwigMetadata {
    id: string;
    name: string;
    description: string;
    type: 'procedural' | 'model';
    filename: string | null;
    defaultScale: number;
    preview: {
        thumbnail: string | null;
        description: string;
    };
}
export interface TwigLibrary {
    version: string;
    description: string;
    twigs: TwigMetadata[];
}
export interface TreeGenerationResult {
    structure: TreeStructure;
    gltf?: string;
    metadata: {
        species: string;
        parameters: TreeParameters;
        generatedAt: Date;
    };
}
//# sourceMappingURL=tree.d.ts.map