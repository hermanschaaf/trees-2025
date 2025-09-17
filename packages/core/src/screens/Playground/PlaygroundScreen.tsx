import React, { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { initializeTreeViewer } from '../TreeDesigner/treeViewerCore';
import { TopMenu, MenuSection } from '../../components/TopMenu/TopMenu';
import './PlaygroundScreen.css';

export const PlaygroundScreen: React.FC = () => {
  const navigate = useNavigate();
  const containerRef = useRef<HTMLDivElement>(null);
  const initializedRef = useRef<boolean>(false);
  const [downloadGltf, setDownloadGltf] = useState<(() => void) | null>(null);

  useEffect(() => {
    console.log('PlaygroundScreen: Component mounting');
    if (containerRef.current && !initializedRef.current) {
      console.log('PlaygroundScreen: Initializing tree viewer');
      initializedRef.current = true;
      
      let cleanup: (() => void) | undefined;
      
      const initPromise = initializeTreeViewer(containerRef.current);
      if (initPromise instanceof Promise) {
        initPromise.then((result) => {
          if (result && typeof result === 'object') {
            cleanup = result.cleanup;
            setDownloadGltf(() => result.downloadGltf);
          }
        });
      } else if (initPromise && typeof initPromise === 'object') {
        cleanup = initPromise.cleanup;
        setDownloadGltf(() => initPromise.downloadGltf);
      }
      
      return () => {
        console.log('PlaygroundScreen: Component unmounting, calling cleanup');
        if (cleanup && typeof cleanup === 'function') {
          cleanup();
        }
        initializedRef.current = false;
        setDownloadGltf(null);
      };
    }
  }, []);

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
    <div className="playground-screen">
      <TopMenu sections={menuSections} />
      <div ref={containerRef} className="playground-content" />
    </div>
  );
};