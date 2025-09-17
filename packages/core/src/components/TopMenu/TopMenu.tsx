import React, { useState, useRef, useEffect } from 'react';
import './TopMenu.css';

export interface MenuItem {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}

export interface MenuSection {
  title: string;
  items: MenuItem[];
}

interface TopMenuProps {
  sections: MenuSection[];
}

interface DropdownPosition {
  top: number;
  left: number;
}

export const TopMenu: React.FC<TopMenuProps> = ({ sections }) => {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [dropdownPosition, setDropdownPosition] = useState<DropdownPosition>({ top: 0, left: 0 });
  const menuRef = useRef<HTMLDivElement>(null);
  const buttonRefs = useRef<{ [key: string]: HTMLButtonElement | null }>({});

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setActiveMenu(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const handleMenuClick = (sectionTitle: string) => {
    const button = buttonRefs.current[sectionTitle];
    if (button) {
      const rect = button.getBoundingClientRect();
      setDropdownPosition({
        top: rect.bottom,
        left: rect.left
      });
    }
    setActiveMenu(activeMenu === sectionTitle ? null : sectionTitle);
  };

  const handleItemClick = (item: MenuItem) => {
    item.onClick();
    setActiveMenu(null);
  };

  return (
    <div className="top-menu" ref={menuRef}>
      <div className="top-menu-bar">
        {sections.map((section) => (
          <div key={section.title} className="menu-section">
            <button
              ref={(el) => (buttonRefs.current[section.title] = el)}
              className={`menu-button ${activeMenu === section.title ? 'active' : ''}`}
              onClick={() => handleMenuClick(section.title)}
            >
              {section.title}
            </button>
            {activeMenu === section.title && (
              <div 
                className="menu-dropdown"
                style={{
                  top: dropdownPosition.top,
                  left: dropdownPosition.left
                }}
              >
                {section.items.map((item, index) => (
                  <button
                    key={index}
                    className={`menu-item ${item.disabled ? 'disabled' : ''}`}
                    onClick={() => !item.disabled && handleItemClick(item)}
                    disabled={item.disabled}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};