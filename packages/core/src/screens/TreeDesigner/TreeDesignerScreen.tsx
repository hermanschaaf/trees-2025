import React, { useEffect, useRef } from 'react';
import { initializeTreeViewer } from './treeViewerCore';

export const TreeDesignerScreen: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const initializedRef = useRef<boolean>(false);

  useEffect(() => {
    if (containerRef.current && !initializedRef.current) {
      initializedRef.current = true;
      const cleanup = initializeTreeViewer(containerRef.current);
      
      return () => {
        if (cleanup) {
          cleanup();
        }
      };
    }
  }, []);

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
};