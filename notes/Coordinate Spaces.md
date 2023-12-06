# Sim Map
`DVec2` coordinate.
- A rect defined by minimum and maximum corners.
- The origin is at the center of the rect.
- Each cell is a patch in netlogo.

# Bevy Transform
`Vec2` coordinate (and Z ordering).
- The background image sprite needs to be scaled to fit the screen.
- The camera view will use these coordinates.
- Probably fine to have a Map coordinate equal a Bevy `Transform`.
    - Only a problem if there is precision issues, since Bevy `Transform` uses f32.

# Bevy Screen Space
Pixel `UVec2` coordinate.
- (0, 0) at bottom left corner.

# GIS Coordinate
`DVec2` coordinate.
- Represents the size in real world units, likely km.
- Could be projected to latitude longitude for coordinates on earth.

# Ascii Grid Raster
Pixel `UVec2` coordinate.
- Real world data imported into the simulation.
- Gives an xy grid integer size, square cell sizes in `GIS` coordinates, and the lower left corner coordinate in `GIS` coordinates.

# PNG Raster
Pixel `UVec2` coordinate. 
- Just a size in pixels, need to manually define the size and location in GIS coordinates.

# Data from MayaSim
## dem.asc
elevation
nodata_value indicates water
## temp.asc
- horizontal 7.45833372236
- vertical 8.11666709003
- cellsize 0.00833...
## netlogo
- envelope: (-200, -216) to (200, 216)
- box: 401x433, origin at (0, 0), 173,633 total patches
unintuitive extra patch is the zero lines, -1 to 1 is 3 patches.
- patch size in pixels: 2.6

`set area ( 516,484  / (count land-patches)) ; km^2`
- `= 5.056083640884573` what does this mean?
- `land-patches count = 102,151`
- actual region area is `320,000 km^2` (excludes water?)
- `516,484 km^2` is the entire region in Maya.png

# Need
## Current
1. Cell Grid IndexTuple (UMapCoord2)
2. Grid To Map using Origin and Cell Size (MapSpace)
3. Gis of Map (DMapCoord2) to Gis of Asset (DGisCoord2) (transform origin, and an equalized scale (keep pixels square?))
4. Asset Gis to Grid using Origin and Cell Size (UGridCoord2 or UPixelCoord2)

**The AsciiGrid assets are all using the same GIS coordinate system, and should be named as such.**

## GridCoord
The zero-indexed TupleIndex of images or the cell grid dont need transformations, although I guess you cant index one with the other.

**They should be distinct types, no universal GridCoord** for the same reason why a transformation is needed for the float coords.

## Game
The game world.
All transforms will use this as the basis
- `GameSpace`: the coordinates used by bevy transforms, origin is 0,0 in bevy.
    - Won't define an origin, or scale
- `Vec2`: position in the game space

## Screen
The camera view space
- `ScreenSpace`: a Rect defined in Vec2 of visible area, and the pixel size of the screen
    - lets you figure out the exact size something will appear.
- `ScreenIndex2`: pixel coordinate

## Map
Grid of cells, but also floats for positioning of agents.
- `MapSpace`: Origin offset from `GameSpace`, and scale.
- `MapIndex2`: The cell IndexTuple, origin is the bottom left of map (min of its rect)
- `MapCoord2`: 
### Beyond Rect
if this is a map area, of the simulated region, how to represent that you can leave this area, it can't just be defined by a rect.
or at-least the origin shouldn't be based on a rect min.

## GIS
Space used by data imported from GIS.
These are projected coordinates to a plane, not Lat,Long.
I don't think they represent a real world unit?
- `GeoSpace`
- `GeoCoord2`: position in this space.
- `GeoIndex2`: grid coordinate in the space, 

### AsciiGrid
grid of data, float origin, float cell size
uses the GIS coordinate space and types
