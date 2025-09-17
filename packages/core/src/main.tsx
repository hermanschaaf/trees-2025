import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

console.log('main.tsx: Script loaded');

const rootElement = document.getElementById('root');
console.log('main.tsx: Root element found:', !!rootElement);

if (!rootElement) {
  console.error('main.tsx: Root element not found!');
  document.body.innerHTML = '<div style="background:red;color:white;padding:20px;">ERROR: Root element not found</div>';
} else {
  console.log('main.tsx: Creating React root and rendering App');
  const root = ReactDOM.createRoot(rootElement);
  root.render(<App />);
}