import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TopMenu, MenuSection } from '../../components/TopMenu/TopMenu';
import './ForestDesignerScreen.css';

export const ForestDesignerScreen: React.FC = () => {
  const navigate = useNavigate();

  const menuSections: MenuSection[] = [
    {
      title: 'File',
      items: [
        {
          label: 'Export Forest',
          onClick: () => console.log('Export Forest clicked'),
          disabled: true // Disabled until forest functionality is implemented
        },
        {
          label: 'Exit',
          onClick: () => navigate('/')
        }
      ]
    },
    {
      title: 'View',
      items: [
        {
          label: 'Reset Camera',
          onClick: () => console.log('Reset Camera clicked'),
          disabled: true
        }
      ]
    }
  ];

  return (
    <div className="forest-designer-screen">
      <TopMenu sections={menuSections} />
      <div className="forest-designer-content">
        <div className="forest-placeholder">
          <h2>Forest Designer</h2>
          <p>Coming soon...</p>
        </div>
      </div>
    </div>
  );
};