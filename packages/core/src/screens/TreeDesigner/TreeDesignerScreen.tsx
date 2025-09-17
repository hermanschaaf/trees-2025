import React, { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { initializeSimpleTreeViewer } from './simpleTreeViewer';
import { TopMenu, MenuSection } from '../../components/TopMenu/TopMenu';
import { TreeTypePanel, TreeType } from '../../components/TreeTypePanel/TreeTypePanel';
import './TreeDesignerScreen.css';

export const TreeDesignerScreen: React.FC = () => {
  const navigate = useNavigate();
  const containerRef = useRef<HTMLDivElement>(null);
  const initializedRef = useRef<boolean>(false);
  const [downloadGltf, setDownloadGltf] = useState<(() => void) | null>(null);
  const [selectedTreeType, setSelectedTreeType] = useState<TreeType>('fir');
  const [currentCleanup, setCurrentCleanup] = useState<(() => void) | null>(null);
  const [regenerateScene, setRegenerateScene] = useState<(() => void) | null>(null);

  const initializeViewer = async (treeType: TreeType) => {
    if (!containerRef.current) return;
    
    // Clean up previous instance
    if (currentCleanup) {
      currentCleanup();
      setCurrentCleanup(null);
      setDownloadGltf(null);
      setRegenerateScene(null);
    }
    
    console.log('TreeDesignerScreen: Initializing tree viewer for', treeType);
    
    const initPromise = initializeSimpleTreeViewer(containerRef.current, treeType);
    if (initPromise instanceof Promise) {
      const result = await initPromise;
      if (result && typeof result === 'object') {
        setCurrentCleanup(() => result.cleanup);
        setDownloadGltf(() => result.downloadGltf);
        setRegenerateScene(() => result.regenerateScene);
      }
    } else if (initPromise && typeof initPromise === 'object') {
      setCurrentCleanup(() => initPromise.cleanup);
      setDownloadGltf(() => initPromise.downloadGltf);
      setRegenerateScene(() => initPromise.regenerateScene);
    }
  };

  useEffect(() => {
    console.log('TreeDesignerScreen: Component mounting');
    if (containerRef.current && !initializedRef.current) {
      initializedRef.current = true;
      initializeViewer(selectedTreeType);
      
      return () => {
        console.log('TreeDesignerScreen: Component unmounting, calling cleanup');
        if (currentCleanup) {
          currentCleanup();
        }
        initializedRef.current = false;
        setDownloadGltf(null);
      };
    }
  }, []);

  // Update scene when tree type changes (without reinitializing)
  useEffect(() => {
    if (initializedRef.current && regenerateScene) {
      console.log('Tree type changed, regenerating scene for:', selectedTreeType);
      regenerateScene(selectedTreeType);
    }
  }, [selectedTreeType]);

  const handleTreeTypeChange = (treeType: TreeType) => {
    console.log(`Tree type changed to: ${treeType}`);
    setSelectedTreeType(treeType);
    // TODO: Apply tree type presets to the tree generation
  };

  const menuSections: MenuSection[] = [
    {
      title: 'File',
      items: [
        {
          label: 'Export GLTF',
          onClick: () => downloadGltf && downloadGltf(),
          disabled: !downloadGltf
        },
        {
          label: 'Exit',
          onClick: () => navigate('/')
        }
      ]
    }
  ];

  return (
    <div className="tree-designer-screen">
      <TopMenu sections={menuSections} />
      <TreeTypePanel
        selectedTreeType={selectedTreeType}
        onTreeTypeChange={handleTreeTypeChange}
      />
      <div ref={containerRef} className="tree-designer-content" />
    </div>
  );
};