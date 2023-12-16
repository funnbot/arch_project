## forager agents
- movement
	- within vision radius search for resources
	- form paths through the patches that allow for decreased energy when moving
	- metric
		- prioritize resource cells, with a higher resource count
		- then prioritize lower travel cost
		- prioritize resource cells that have fewer other foragers already on them
- energy
	- depleted when moving
	- regained when collecting resources
		- takes time to get energy from the resource (metabolism)
## resource patches
- regenerate or not?
- have a size,
- multiple types of resources?
## searching for stable population
- allow new foragers to spawn over time