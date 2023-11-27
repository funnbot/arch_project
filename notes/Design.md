# Exclusive Sim Schedule
`SimUpdate`
Probably need a `SystemSet` to order the read world, decision, and write to world systems.
- `SimSet::Read`
- `SimSet::Decide`
- `SimSet::Write`
Or don't need a `SystemSet`, the ordering of each system could be explicit.


an exclusive system which runs the exclusive, multi-threaded, schedule.
This schedule is very similar to the `FixedUpdateSchedule`, but is designed for a timestep which changes. 
Specialize the `Time<>` resource, and copy the code from `FixedUpdateSchedule`.
- If possible, the `SimUpdate` schedule may be set to process multi day timesteps, which the `Time<>` resource may not be designed for.
- This needs to be a new schedule, and not `FixedUpdate`, to allow for an auto adjusting algorithm, which accumulates the processing time of each schedule run loop, and prevents the rest of the app from dipping below a reasonable fps.
- If the schedule is run in an Update system, won't need to flush command buffers.
## Timing
- handle a large scale of time, thousands of years
	- in seconds, max duration of 580,000,000,000 years
	- milli: 580,000,000 years
	- nanos: 580 years
- **durations will be stored as seconds**
- precision only needs to be down to the day at most.
- yet still need to be able to convert to a standard duration in seconds for calculations
- handle changing the timestep dynamically.
- handle counting the real elapsed milliseconds to prevent spending too long iterating.
- need another system to dynamically set bevy fps, and then this time can read that, and give more leeway for number of steps.
- for the most part, the current simulation works on ticks, so the time isn't used for calculations.
- however, the time will be used when displaying the calculations.
- need to be able to calculate off of an epoch, durations are positive, this is only if the year is displayed though.
- needs to be accessible and correct, before and after the exclusive system runs.
- the time it is tracking is completely independent
- to calculate the maximum allowed simulation time
	- cant be greater than the minimum frametime - the app delta
### Each Loop
- run the schedule, all systems will get the same time value
	- 

# Systems
## Agents
In `SimUpdate` schedule.
- read the surrounding environment and save to self
	- heightmap
	- terrain
	- weather
- read nearby agents and save to self
	- in non-grid simulations, needs read only access to spatial data structure that stores all agent locations.
So much is saved to each individual agent, need to limit this, maybe combine the read and decide systems...
If a system can access its neighbors, then they need to not be writing to themselves.
- decide, using saved info about environment and neighbors.
- write decision, apply changes to self and environment.
	- or just to environment, if read and decide are combined.

