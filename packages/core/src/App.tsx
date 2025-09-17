import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { SplashScreen } from './screens/SplashScreen/SplashScreen';
import { TreeDesignerScreen } from './screens/TreeDesigner/TreeDesignerScreen';
import { ForestDesignerScreen } from './screens/ForestDesigner/ForestDesignerScreen';
import { PlaygroundScreen } from './screens/Playground/PlaygroundScreen';
import './App.css';

function App() {
  console.log('App: Rendering');
  
  return (
    <Router>
      <div className="app">
        <Routes>
          <Route path="/" element={<SplashScreen />} />
          <Route path="/tree-designer" element={<TreeDesignerScreen key="tree-designer" />} />
          <Route path="/forest-designer" element={<ForestDesignerScreen />} />
          <Route path="/playground" element={<PlaygroundScreen key="playground" />} />
        </Routes>
      </div>
    </Router>
  );
}

export default App;