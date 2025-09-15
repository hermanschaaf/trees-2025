export type ScreenType = 'tree-designer' | 'forest-viewer';
export interface ScreenConfig {
    id: ScreenType;
    name: string;
    description: string;
    icon: string;
    tools: ToolConfig[];
}
export interface ToolConfig {
    id: string;
    name: string;
    description: string;
    icon: string;
    category: 'generation' | 'modification' | 'visualization' | 'export';
    parameters?: ToolParameter[];
}
export interface ToolParameter {
    id: string;
    name: string;
    type: 'slider' | 'select' | 'toggle' | 'color' | 'text';
    min?: number;
    max?: number;
    step?: number;
    options?: {
        value: any;
        label: string;
    }[];
    defaultValue: any;
}
//# sourceMappingURL=screen.d.ts.map