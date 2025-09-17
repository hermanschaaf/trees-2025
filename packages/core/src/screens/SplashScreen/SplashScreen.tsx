import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import './SplashScreen.css';

export const SplashScreen: React.FC = () => {
  const navigate = useNavigate();

  useEffect(() => {
    console.log('SplashScreen: Component mounted');
    return () => {
      console.log('SplashScreen: Component unmounting');
    };
  }, []);

  const handleNewTree = () => {
    navigate('/tree-designer');
  };

  const handleNewForest = () => {
    navigate('/forest-designer');
  };

  const handlePlayground = () => {
    navigate('/playground');
  };

  console.log('SplashScreen: Rendering component');
  
  return (
    <div className="splash-screen">
      <div className="splash-content">
        <h1 className="splash-title">3D Tree Generator</h1>
        <p className="splash-subtitle">Create beautiful procedural trees and forests</p>
        
        <div className="splash-options">
          <button 
            className="splash-option-btn"
            onClick={handleNewTree}
          >
            <div className="option-icon">🌳</div>
            <h3>New Tree</h3>
            <p>Design and customize a single tree</p>
          </button>
          
          <button 
            className="splash-option-btn"
            onClick={handleNewForest}
          >
            <div className="option-icon">🌲</div>
            <h3>New Forest</h3>
            <p>Create and manage a forest of trees</p>
          </button>
          
          <button 
            className="splash-option-btn"
            onClick={handlePlayground}
          >
            <div className="option-icon">🛝</div>
            <h3>Playground</h3>
            <p>Experiment with advanced tree controls</p>
          </button>
        </div>
      </div>
    </div>
  );
};