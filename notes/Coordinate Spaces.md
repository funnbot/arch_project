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
- Just a size in pixels, need to define the size and location in GIS coordinates.

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

