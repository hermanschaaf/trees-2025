import React, { useState, useRef, useEffect } from 'react';
import './TreeTypePanel.css';

export type TreeType = 'fir' | 'birch' | 'ivy';

interface TreeTypeOption {
  id: TreeType;
  name: string;
  description: string;
  icon: string;
}

interface TreeTypePanelProps {
  selectedTreeType: TreeType;
  onTreeTypeChange: (treeType: TreeType) => void;
}

const treeTypes: TreeTypeOption[] = [
  {
    id: 'fir',
    name: 'Fir',
    description: 'Evergreen coniferous tree with needle-like leaves',
    icon: '🌲'
  },
  {
    id: 'birch',
    name: 'Birch',
    description: 'Deciduous tree with distinctive white bark',
    icon: '🌳'
  },
  {
    id: 'ivy',
    name: 'Ivy',
    description: 'Climbing vine with distinctive lobed leaves',
    icon: '🍃'
  }
];

export const TreeTypePanel: React.FC<TreeTypePanelProps> = ({
  selectedTreeType,
  onTreeTypeChange
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const selectedOption = treeTypes.find(type => type.id === selectedTreeType);

  const handleOptionClick = (treeType: TreeType) => {
    onTreeTypeChange(treeType);
    setIsOpen(false);
  };

  return (
    <div className="tree-type-dropdown" ref={dropdownRef}>
      <button 
        className="tree-type-trigger"
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className="tree-type-trigger-icon">{selectedOption?.icon}</span>
        <span className="tree-type-trigger-text">{selectedOption?.name}</span>
        <span className="tree-type-trigger-arrow">{isOpen ? '▲' : '▼'}</span>
      </button>
      
      {isOpen && (
        <div className="tree-type-dropdown-menu">
          {treeTypes.map((treeType) => (
            <button
              key={treeType.id}
              className={`tree-type-dropdown-item ${
                selectedTreeType === treeType.id ? 'selected' : ''
              }`}
              onClick={() => handleOptionClick(treeType.id)}
            >
              <span className="tree-type-item-icon">{treeType.icon}</span>
              <div className="tree-type-item-info">
                <div className="tree-type-item-name">{treeType.name}</div>
                <div className="tree-type-item-description">{treeType.description}</div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};