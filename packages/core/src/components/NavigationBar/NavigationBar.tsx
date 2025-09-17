import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import './NavigationBar.css';

export const NavigationBar: React.FC = () => {
  const navigate = useNavigate();
  const [showFileMenu, setShowFileMenu] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const handleBackToHome = () => {
    console.log('NavigationBar: Navigating to home');
    navigate('/');
    setShowFileMenu(false);
  };

  const handleNewTree = () => {
    console.log('NavigationBar: Navigating to tree designer');
    navigate('/tree-designer');
    setShowFileMenu(false);
  };

  const handleNewForest = () => {
    console.log('NavigationBar: Navigating to forest designer');
    navigate('/forest-designer');
    setShowFileMenu(false);
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowFileMenu(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  return (
    <nav className="navigation-bar">
      <div className="nav-brand">
        <button onClick={handleBackToHome} className="brand-btn">
          3D Tree Generator
        </button>
      </div>
      
      <div className="nav-menu">
        <div className="nav-dropdown" ref={dropdownRef}>
          <button 
            className="nav-menu-btn"
            onClick={() => setShowFileMenu(!showFileMenu)}
          >
            File ▼
          </button>
          {showFileMenu && (
            <div className="nav-dropdown-content">
              <button onClick={handleNewTree}>New Tree</button>
              <button onClick={handleNewForest}>New Forest</button>
              <hr />
              <button onClick={handleBackToHome}>Back to Home</button>
            </div>
          )}
        </div>
      </div>
    </nav>
  );
};