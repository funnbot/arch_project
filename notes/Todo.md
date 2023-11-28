- [ ] Camera controls
	- [ ] move the camera around
		- [ ] using mouse
		- [ ] using keyboard
	- [ ] zoom in and out
- [ ] Be specific about the camera view rect and the map rect (which is split up by cells, and is accessible as a grid).
- [ ] How to add a plugin to the SimUpdate schedule without importing it?
	- Pass in the `ScheduleLabel` some how.
	- Although, most plugins define their own schedule if they require it, `FixedUpdate` is special case, and is accessed globally.
- [ ] create `sim` folder for simulation code.
- [ ] design
	- [ ] create a system that loads all of the ascii grid data and stores the ids in a resource
		- [ ] system that reads asset loading events to check when the data is loaded.
	- [ ] use AppStates for loading and begin simulation
	- [ ] system that applies the GIS data to the map.
	- [ ] system that renders the map with different options, 