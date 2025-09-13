# Twig Library

This directory contains the GLTF twig library for the 3D tree generator.

## Structure

- `twigs-library.json` - Metadata file containing information about all available twig types
- `*.gltf` files - GLTF models for each twig type (to be provided by user)
- `*-thumb.jpg` files - Optional thumbnail images for twig previews

## Supported Twig Types

The library currently supports the following twig types as defined in `twigs-library.json`:

1. **Procedural Twigs** (default) - Generated using basic Three.js geometries
2. **Basic Leaf Cluster** - Simple cluster of leaves
3. **Small Branch with Leaves** - Thin branch with scattered leaves  
4. **Flower Bud** - Small colorful flower bud for branch tips
5. **Pine Needle Cluster** - Bundle of sharp pine needles for coniferous trees
6. **Autumn Leaves** - Colorful autumn foliage in various colors

## Adding New Twigs

To add a new twig type:

1. Add the GLTF file to this directory
2. Update `twigs-library.json` with the new twig metadata
3. Optionally add a thumbnail image

## Usage

Twig types can be selected in the frontend using the "Twig Library" GUI controls. The system will:

- Load and cache GLTF models
- Apply proper positioning, scaling, and rotation
- Fallback to procedural generation if GLTF loading fails
- Enable shadows and lighting for GLTF models

## File Format

GLTF files should be optimized for web use and include:
- Appropriate scaling (models will be further scaled by the `defaultScale` parameter)
- Proper material definitions
- Efficient polygon counts for real-time rendering