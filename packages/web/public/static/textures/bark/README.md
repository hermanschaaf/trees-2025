# Bark Texture Library

This directory contains the bark texture library for the 3D tree generator.

## Structure

- `bark-library.json` - Metadata file containing information about all available bark textures
- `*_diffuse.jpg` - Diffuse/albedo textures (color maps)
- `*_normal.jpg` - Normal maps for surface detail
- `*_roughness.jpg` - Roughness maps for material properties

## Supported Bark Types

The library currently supports the following bark types as defined in `bark-library.json`:

1. **Default (No Texture)** - Solid color material without texture
2. **Oak Bark** - Rough textured oak tree bark with deep grooves
3. **Birch Bark** - Smooth white birch bark with dark horizontal lines
4. **Pine Bark** - Scaly pine bark with reddish-brown coloration
5. **Cherry Bark** - Smooth cherry bark with horizontal lenticels
6. **Willow Bark** - Deeply furrowed willow bark with vertical ridges
7. **Maple Bark** - Smooth to slightly furrowed maple bark

## Adding New Bark Textures

To add a new bark type:

1. **Add texture files** to this directory:
   - `new_bark_diffuse.jpg` - Color/albedo map (required)
   - `new_bark_normal.jpg` - Normal map (optional but recommended)
   - `new_bark_roughness.jpg` - Roughness map (optional)

2. **Update `bark-library.json`** with the new bark metadata:
```json
{
  "id": "new-bark",
  "name": "New Bark Type",
  "description": "Description of the bark texture",
  "type": "texture",
  "diffuse": "new_bark_diffuse.jpg",
  "normal": "new_bark_normal.jpg", 
  "roughness": "new_bark_roughness.jpg",
  "scale": 1.0,
  "tiling": [4.0, 8.0],
  "color": "#8B4513"
}
```

## Texture Requirements

### File Formats
- **Supported**: JPG, PNG, WebP
- **Recommended**: JPG for diffuse and roughness, PNG for normal maps
- **Resolution**: 512x512 to 2048x2048 pixels

### Texture Properties
- **Diffuse Maps**: Should contain the bark color information
- **Normal Maps**: Should be in tangent space, RGB format
- **Roughness Maps**: Grayscale images (black = smooth, white = rough)

### UV Mapping
- Textures are applied using cylindrical mapping
- The `tiling` parameter controls repeat frequency [U, V]
- Higher V values create more vertical repetition (good for vertical bark patterns)

## Usage

Bark textures can be selected in the frontend using the "Bark Textures" GUI controls. The system will:

- Load and cache texture files automatically
- Apply proper UV tiling and scaling
- Handle PBR material properties (metalness, roughness, normal scaling)
- Fallback to solid colors if textures fail to load
- Update materials in real-time without rebuilding the tree mesh

## Technical Notes

### Material System
- Uses Three.js `MeshStandardMaterial` for PBR rendering
- Supports diffuse, normal, and roughness texture channels
- Automatic texture caching for performance
- Proper memory management and cleanup

### Performance
- Textures are loaded asynchronously and cached
- Materials are shared where possible
- Mipmapping enabled for smooth scaling
- Linear filtering for best quality

### Color Space
- Diffuse textures assumed to be in sRGB color space
- Normal maps in linear space
- Proper color space handling by Three.js

## Troubleshooting

**Textures not loading:**
- Check file paths in `bark-library.json`
- Ensure texture files exist in the correct directory
- Check browser console for loading errors

**Poor texture quality:**
- Increase texture resolution
- Check UV tiling parameters
- Ensure proper normal map format (tangent space)

**Performance issues:**
- Reduce texture resolution
- Use compressed texture formats (JPG instead of PNG where possible)
- Consider texture atlasing for multiple similar bark types