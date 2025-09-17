// Web platform entry point
console.log('web/main.ts: Loading web entry point');

try {
  import('@trees/core/main.tsx');
  console.log('web/main.ts: Successfully imported core/main.tsx');
} catch (error) {
  console.error('web/main.ts: Error importing core/main.tsx:', error);
  document.body.innerHTML = '<div style="background:red;color:white;padding:20px;">ERROR: Failed to import core React app</div>';
}