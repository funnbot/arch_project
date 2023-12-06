;    MayaSim: An agent-based model of the ancient Maya social-ecological system.
;    Copyright (C) 2013  Scott Heckbert
;
;    scott.heckbert@gmail.com

extensions [ gis ]

;##############       SETUP        ##################         SETUP       ##################

globals[
  mask-dataset
  elevation-dataset
  soils-dataset
  temp-dataset
  precip-dataset
  land-patches
  vacant-lands
  traders
  border
  visited-nodes
  network-start
  failed-cities
  crop1-yield
  climate-cycle-counter
  abandoned-crops
  new-crops
  total-migrant-population
  giant-component-size
  component-size
  giant-start-node
  search-completed
  origin-city
  area
  mx
  mn
  total-migrant-utility
  Rainfall-Change]

patches-own [
  original-rainfall
  rainfall
  temp
  elevation
  soil-prod
  slope
  flow
  pop-gradient
  env-degrade
  npp
  yield
  ag-suit
  BCA-ag
  is-ag
  ag-impact
  forest-state
  succession-counter
  travel-cost
  overland-TC
  freshwater-TC
  cropping-value
  water-value
  forest-food-value
  rain-value
  ecosystem-services-value
  is-vacant
  patch-migrant-utility
  Travel-Cost-ut
  ES-ut
  my-settlement
  is-land-patch
  is-border]

breed [ settlements settlement]

settlements-own [
  birthrate
  trade-strength
  centrality
  cluster-number
  age
  population
  gdp-per-cap
  trade-GDP
  yield-GDP
  ecoserv-GDP
  death-rate
  out-migration
  out-migration-rate
  settlement-yield
  ecoserv-benefit
  my-ag-patches
  my-influence-patches
  rank
  trade-benefit
  explored?
  city-travel-cost  ]

breed [ raindrops raindrop]
raindrops-own[ rain-volume]
breed [ searchers searcher]
searchers-own[ location path-cost ]
links-own [   trade-flow  ]
breed [ migrants migrant ]
migrants-own [
  migrant-population
  mig-TC-pref
  mig-ES-pref
  my-migrant-location
  my-pioneer-patch
  pioneer-set
  my-migrant-utility
  parent]

to startup
 ;Combines the effects of clear-globals, clear-ticks, clear-turtles, clear-patches, clear-drawing, clear-all-plots, and clear-output
  clear-all
  reset-ticks
  setup-gis
  climate-scenario
  rain
  calc-npp
  calc-travel-cost
  reset
end

to setup-gis
  import-pcolors-rgb "Maya.png"
  set elevation-dataset gis:load-dataset "dem.asc"
  set soils-dataset gis:load-dataset "soil.asc"
  set temp-dataset gis:load-dataset "temp.asc"
  set precip-dataset gis:load-dataset "precip.asc"
  gis:set-world-envelope (gis:envelope-union-of
    (gis:envelope-of elevation-dataset)
    (gis:envelope-of soils-dataset)
    (gis:envelope-of temp-dataset)
    (gis:envelope-of precip-dataset)             )
  gis:apply-raster elevation-dataset elevation
  gis:apply-raster soils-dataset soil-prod
  gis:apply-raster temp-dataset temp
  gis:apply-raster precip-dataset original-rainfall
  ; the (x <= 0) or (x >= 0) seems to be a pattern for null or NaN checking.
  ; assuming this is for fixing locations with the nodata_value
  ;
  ; ifelse cond [ true block ] [ false block ]
  ask patches [  
      ifelse (soil-prod <= 0) or (soil-prod >= 0)   [ ] [ set soil-prod 1.5   ] 
      if elevation <= 1 [set soil-prod 1.5]  if soil-prod >= 6 [  set soil-prod 1.5]    
  ]
  ask patches [  
    ifelse (elevation <= 0) or (elevation >= 0)  [ set is-land-patch 1] [ set elevation 0 ]  
  ]
  ask patches [ 
    ifelse (temp <= 0) or (temp >= 0)  [  ] [ set temp 21 * 12 ]  
  ]
  ask patches [  
    ifelse (original-rainfall <= 0) or (original-rainfall >= 0) 
      [ ] 
      [ set original-rainfall 1200 ]  
  ]
  set land-patches patches with [ is-land-patch  = 1 ]
  set border (land-patches with [ (sum [is-land-patch] of neighbors < 6) ])
  ask border [set is-border 1]
  set area ( 516484  / (count land-patches)) ; Km2
  ask land-patches [
    set rainfall original-rainfall
    set temp temp / 12
  ]
  ; Tells each patch to give equal shares of (number * 100) percent of the value of patch-variable to its eight neighboring patches. number should be between 0 and 1. Regardless of topology the sum of patch-variable will be conserved across the world. (If a patch has fewer than eight neighbors, each neighbor still gets an eighth share; the patch keeps any leftover shares.) 
  repeat  max-pxcor / 20 [diffuse  soil-prod 0.9]
  calculate-slope
end

to reset
  clear-all-plots
  reset-ticks
  ask turtles [die]
  setup-patches
  if Humans [ setup-settlements ]
  set climate-cycle-counter 0
  set Rainfall-Change 0
  update-view
end

to setup-patches
  set-default-shape turtles "circle"
  ask land-patches [
    set is-ag 0
    set pop-gradient 0
    set ag-impact 0
    set env-degrade 0
    set succession-counter  random npp / 3
    set forest-state 1  if succession-counter > state-change-s2 [ set forest-state 2 ]  if succession-counter > state-change-s3  [ set forest-state 3  ]]
end

to setup-settlements
  set-default-shape settlements "mayahouse"
  ask n-of num-cities land-patches [
    sprout-settlements 1[
      set color grey
      set rank 4
      set population  1000 + random 1000
      set death-rate  random-float 0.1 + 0.05
      set gdp-per-cap  random-float 5  + 5   ]]
  ask land-patches with [count settlements-here > 0] [ set my-settlement  ( one-of settlements-here)   ]
  ask settlements [  area-of-influence  set my-ag-patches ( my-influence-patches with [my-settlement = myself])  ask my-ag-patches [   set is-ag 1]  ]
end

to calculate-slope
  ; convole <dataset> <rows> <cols> <matrix> <centerRow> <centerCol>
  let horizontal-gradient gis:convolve elevation-dataset 3 3 [ 1 0 -1 2 0 -2 1 0 -1 ] 1 1
  let vertical-gradient gis:convolve elevation-dataset 3 3 [ 1 2 1 0 0 0 -1 -2 -1 ] 1 1
  let gradient gis:create-raster gis:width-of elevation-dataset gis:height-of elevation-dataset gis:envelope-of elevation-dataset
  let x 0
  repeat (gis:width-of gradient)
  [ let y 0
    repeat (gis:height-of gradient)
    [ let gx gis:raster-value horizontal-gradient x y
      let gy gis:raster-value vertical-gradient x y
      if ((gx <= 0) or (gx >= 0)) and ((gy <= 0) or (gy >= 0))
      [ gis:set-raster-value gradient x y sqrt ((gx * gx) + (gy * gy)) ]
      set y y + 1 ]
    set x x + 1 ]
  let min-g gis:minimum-of gradient
  let max-g gis:maximum-of gradient
  gis:apply-raster gradient slope
  ask land-patches [
    ; If number is positive, reports a random integer greater than or equal to 0, but strictly less than number.
    ifelse (slope <= 0) or (slope >= 0)  [ ] [ set slope (5 + random 10 ) ]
    set slope log (slope + 1) 2
  ]
end

;##############       RUN        ##################         RUN       ##################

to run-model
  if Nature [
    climate-scenario
    if raining [
      rain]
    calc-npp
    calc-pop-gradient
    forest-succession
    calc-ecosystem-services
    calc-soil-degradation
    calc-BCA-Ag
    Agriculture ]
  if Humans [
    Demographics
    if trade and ticks >= 20 [
      Build-routes
      calc-travel-cost
      trading]
    recalc-gdp ]
  update-view
  tick
  if ticks = 400 [stop] ; %
  if ticks = 1 [clear-all-plots]
end

;#############         MODEL FUNCTIONS         ##################         MODEL FUNCTIONS         ##################

to climate-scenario
  if Climate-Cycle [
    set climate-cycle-counter  climate-cycle-counter +  1
    if climate-cycle-counter = Climate-var * 8 [set Rainfall-Change Rainfall-Change - rain-change ]
    if climate-cycle-counter = Climate-var * 7 [set Rainfall-Change Rainfall-Change + rain-change ]
    if climate-cycle-counter = Climate-var * 6 [set Rainfall-Change Rainfall-Change + rain-change ]
    if climate-cycle-counter = Climate-var * 5 [set Rainfall-Change Rainfall-Change + rain-change ]
    if climate-cycle-counter = Climate-var * 4 [set Rainfall-Change Rainfall-Change + rain-change ]
    if climate-cycle-counter = Climate-var * 3 [set Rainfall-Change Rainfall-Change - rain-change]
    if climate-cycle-counter = Climate-var * 2 [set Rainfall-Change Rainfall-Change - rain-change ]
    if climate-cycle-counter = Climate-var * 1 [set Rainfall-Change Rainfall-Change - rain-change ]
    if climate-cycle-counter >= Climate-var * 8 [set climate-cycle-counter 0  ]]
  if Climate-Change [
    ask land-patches [
      let rainfall-multiplier  (1 + Rainfall-Change  ) ; 1 +  max-dist-fulcrum / distance-to-fulcrum *
      let cleared-rainfall-effect (( count (neighbors with [forest-state = 1]  ) * Veg-Rainfall ))
      set rainfall (  original-rainfall  * rainfall-multiplier)  - cleared-rainfall-effect]]
end

to rain
  ask raindrops [die]
  ask land-patches [
    set flow 0]
  ask n-of ((count land-patches) * precip-percent) land-patches [
    sprout-raindrops 1 [
      set color blue
      set rain-volume ( rainfall / 1000000 / precip-percent * 10) ; cubic km2  # conversion
      set size rain-volume * 30  ]]
  repeat rain-steps   [
    ask raindrops [
      if random-float 100 < Infitration [die]
      let target min-one-of neighbors [ elevation + ((sum [rain-volume] of raindrops-here) ) ]
      ifelse [elevation + ((sum [rain-volume] of raindrops-here) )] of target < elevation + ((sum [rain-volume] of raindrops-here) )[
        set color blue
        move-to target
        ask patch-here [ set flow (flow + ([rain-volume] of myself ) )]
        if [is-land-patch] of patch-here = 0 or [is-border] of patch-here = 1 [die] ][
        set color green ]]] ; and don't count flow... ;
  ask raindrops [die]
end

to calc-npp ;
  ask land-patches [
    let npp-rain (  3000 * ( 1 - exp( -0.000664 * rainfall))  )
    let npp-temp (  3000 / ( 1 + exp( 1.315 - (0.119 * temp))))
    ifelse (npp-rain < npp-temp) [  set npp ( npp-rain ) ][ set npp ( npp-temp ) ] if npp < 500 [set npp 500]]
end

to forest-succession
  let mean-npp (mean [npp] of land-patches)
  let interval 4
  repeat interval [
    ask land-patches with [is-ag = 1][  set forest-state 1 set succession-counter 1 ]
    ask land-patches with [is-ag = 0][
      set succession-counter (succession-counter + 1 )
      let npp-multiplier   (npp / mean-npp )
      if random-float 100 < disturb-rate * (pop-gradient * 2 + 1 ) [
        if forest-state = 2 [  set forest-state  1  set succession-counter 1 ]
        if forest-state = 3 [  set forest-state  2  set succession-counter state-change-s2]]
      if forest-state = 1 and state-change-s2 / npp-multiplier <= succession-counter  [  set forest-state 2  set succession-counter state-change-s2 ]
      if forest-state = 2 and state-change-s3 / npp-multiplier <= succession-counter and (count neighbors with [forest-state = 3 ] >= s3num-neigh) [  set forest-state 3  ] ] ]
end

to Agriculture
  set failed-cities 0
  set abandoned-crops 0
  set new-crops 0
  ask land-patches [
    set is-vacant 1
    set my-settlement 0    ]
  ask settlements [
    if count my-ag-patches = 0 [ set failed-cities  failed-cities + 1 die]
    ask my-ag-patches [   set my-settlement myself]]
  let count-ouside count land-patches with [is-ag = 1 and my-settlement = 0 ]
  set abandoned-crops abandoned-crops + count-ouside
  ask land-patches with [is-ag = 1 and my-settlement = 0 ] [set is-ag 0 set yield 0 ]
  ask settlements [    area-of-influence  ]
  ask settlements with [ count my-ag-patches > 0 ][
    let ag-pop-density (population / count my-ag-patches / area)
    if ag-pop-density < 40 and age > 5   [
      repeat  ceiling ( 30 / ag-pop-density  )  [
        ask min-one-of my-ag-patches [    (BCA-ag - (ag-travel-cost * sqrt area * distance myself) /  ([sqrt population] of myself)  )  ] [  ; ^
          set is-ag 0                set my-settlement 0         set yield 0
          set abandoned-crops abandoned-crops + 1  ]]]
    let newest-crops  ( ag-pop-density / 125 )
    repeat floor newest-crops [
      let ag-search-list my-influence-patches with [ is-ag = 0 and  (BCA-ag - (ag-travel-cost * (sqrt area * distance myself)) /  ([sqrt population] of myself)  )  > 0] ; ^
      ifelse count ag-search-list > 0 [
        ask max-one-of ag-search-list [( BCA-ag - (ag-travel-cost * (sqrt area * distance myself)) /  ([sqrt population] of myself)  )  ][; ^
          set my-settlement ( myself )
          set is-ag 1
          set new-crops new-crops + 1   ]][]]]
  ask settlements [ ; abandon crops
    ask my-ag-patches with [ (BCA-ag - (ag-travel-cost * sqrt area * (distance myself)) /  ([sqrt population] of myself ) ) < 0 ] [; ^
      set is-ag 0      set my-settlement 0       set yield 0
      set abandoned-crops abandoned-crops + 1]]
  ask settlements [
    set my-ag-patches ( my-influence-patches with [ my-settlement = myself])
    ask my-influence-patches [set is-vacant 0]]
  ask settlements [
    set settlement-yield 0
    ask my-ag-patches [
      set yield (max-yield * (1 - origin-shift * ( exp (slope-yield * ag-suit)  )))
      if yield <= 0 [set yield 1]   ]]
  ask settlements with [count my-ag-patches > 0] [
    set settlement-yield ( mean [yield] of my-ag-patches )  ]
end

to calc-soil-degradation
  ask settlements [  ask my-ag-patches [  set ag-impact ag-impact + soil-deg-rate ]]
  ask land-patches [  if forest-state >= 3 [  set ag-impact ag-impact - soil-regen-rate ]  if ag-impact < 0 [ set ag-impact 0 ] ]
end

to calc-BCA-Ag
  ask land-patches [
    set ag-suit ( ag-suit-npp * npp  - ag-suit-slope * slope  -  ag-suit-flow * flow  +  ag-suit-soils * soil-prod - ag-impact)
    if ag-suit > 650 [ set ag-suit 650]   ]
  ask land-patches [
    set BCA-ag (max-yield * (1 - origin-shift * ( exp (slope-yield * ag-suit) )) )- estab-cost]
end

to calc-crop-yield
  set  crop1-yield []   let crop-counter 1
  repeat 1000 [
    let yield-c1 (max-yield * (1 - origin-shift * ( exp (slope-yield * crop-counter )   )))
    if yield-c1 < 0 [set yield-c1 0]     set  crop1-yield lput yield-c1 crop1-yield     set crop-counter (crop-counter + 1)   ]
  ; set-current-plot "Crop-Production"  clear-plot   set-current-plot-pen "yield-crop1"  foreach sort-by [ ?1 < ?2 ] crop1-yield [  plot ? ] ;; is this necessary in netlogo 6?
end

to calc-ecosystem-services
  ask land-patches [
    set water-value  flow  * flow-value-param
    ; set rain-value (rainfall / 1000 * precip-value-param) this was originally commented out
    set cropping-value crop-value-param * ag-suit
    set forest-food-value  forest-value-param * (forest-state - 1)
    ; set env-degrade pop-gradient  * ES-deg-factor  this was originally commented out
    set ecosystem-services-value (  cropping-value  +  water-value  + forest-food-value   ) ; - env-degrade   ; rain-value  +
    if  ecosystem-services-value > 250 [ set ecosystem-services-value 250 ]   if  ecosystem-services-value < 1 [ set ecosystem-services-value 1 ] ]
end

to Demographics
  ask settlements [
    set birthrate 0.15
    let max-birth-rate 0.15
    let min-birth-rate -0.2
    let shift 0.325
    if population-control and population >= 5000 [ set birthrate ( -((max-birth-rate - min-birth-rate)/ 10000) * population + shift  ) ]
    set age age + 1
    let max-death-rate 0.25
    let min-death-rate 0.005
    set death-rate ( -((max-death-rate - min-death-rate)/ 1) * gdp-per-cap + max-death-rate  )
    set death-rate  ( precision death-rate 3 )
    if death-rate <= min-death-rate [set death-rate min-death-rate]
    if death-rate >= max-death-rate [set death-rate max-death-rate]
    let max-mig-rate 0.15
    let min-mig-rate 0
    set out-migration-rate (-((max-mig-rate - min-mig-rate)/ 1) *  gdp-per-cap + max-mig-rate )
    if out-migration-rate < min-mig-rate [set out-migration-rate min-mig-rate]
    if out-migration-rate > max-mig-rate [set out-migration-rate max-mig-rate]
    set out-migration-rate  ( precision out-migration-rate 3 )
    set out-migration (out-migration-rate * population)
    let pop-change ((birthrate - death-rate )* population )
    set population int(population + pop-change  )   ]
  set vacant-lands ( land-patches with [ BCA-ag > 0 and is-vacant = 1])
  if migration and count vacant-lands > 300 [migrate]
  ask settlements [  if population <= estab-cost * 0.4    [  ask my-ag-patches [  set my-settlement 0  set is-ag 0]  set failed-cities  failed-cities + 1 die] ] ; #
  ask land-patches [  if count settlements-here > 1 [ ask one-of settlements-here  [ set failed-cities  failed-cities + 1 die]]]
end

to migrate
  ask  settlements with [out-migration > 400] [ ;
    if random 100 <= 50 [
      hatch-migrants 1   [
        set parent one-of settlements-here
        set migrant-population [out-migration] of parent
        ask parent [set population population - out-migration]
        set size 0.5
        set color ([color] of myself)
        set mig-TC-pref  TC-pref
        set mig-ES-pref  ES-pref  ]]]
  set total-migrant-population sum [migrant-population] of migrants
  ask migrants [
    ifelse count vacant-lands > 75 [] [ask parent [set population (population + ([migrant-population] of myself)) ] die]
    set pioneer-set (  n-of 75 vacant-lands ) ; # 15
    ask pioneer-set [
      let distance-to-settlement( sqrt area * distance myself)
      set Travel-Cost-ut ( [mig-TC-pref] of myself  * distance-to-settlement )
      set ES-ut ( [mig-ES-pref] of myself * [ecosystem-services-value] of self )
      set patch-migrant-utility ( Travel-Cost-ut + ES-ut)  ]
    set my-migrant-utility (max [ patch-migrant-utility ] of pioneer-set)
    set my-migrant-location (  max-one-of pioneer-set [patch-migrant-utility]  )
    ; show my-migrant-utility  also originally commented out
    if my-migrant-utility > 0 [      move-to my-migrant-location       ]  ]
  set total-migrant-utility sum [my-migrant-utility] of migrants
  
  ask migrants [
    let neigh-count count turtles-on patches with [sqrt area * (distance myself) <= 7.5]; neighbors
    if neigh-count > 1 [die]
    if count turtles-here > 1 [die]
    if count settlements-here = 0 [
      ask patch-here [
        sprout-settlements 1[
          set rank 4
          set population ([migrant-population] of one-of migrants-here)
          set color [color] of one-of migrants-here
          set color  grey
          ask patch-here [ set my-settlement  ( one-of settlements-here)   ]
          set my-ag-patches ( land-patches with [my-settlement = myself])  ask my-ag-patches [  set is-ag 1]
          set my-influence-patches  land-patches with [sqrt area * distance myself <= 2 ]]]  die  ]]
end

to area-of-influence
  let pop-scaled-dist (population ^ 0.8 ) / 60
  set my-influence-patches ( land-patches with [ sqrt area * distance myself  <= pop-scaled-dist])
end

to calc-pop-gradient
  ask land-patches [set pop-gradient 0]
  ask settlements  [
    ask my-influence-patches [
      let dist sqrt area * distance myself
      set pop-gradient (pop-gradient + [population] of myself / (dist + 1) / 300 )
      if  pop-gradient > 15 [set pop-gradient 15]]] ; #
end

to calc-travel-cost
  ask land-patches [
    set travel-cost 40
    set freshwater-TC 0
    set overland-TC ( slope-TC * slope)
    if flow > 0 [ set freshwater-TC flow-TC *  flow   ]
    set travel-cost  ( travel-cost + overland-TC - freshwater-TC )     if travel-cost > 70 [set travel-cost 70]  if travel-cost < 1 [set travel-cost 1]]
  ask settlements [ set city-travel-cost (mean [travel-cost] of my-influence-patches)     ]
end

to Build-routes ;
  ask settlements [
    let start-rank rank
    set rank 4    set shape "mayahouse"
    if population >= rank-3-pop [  set rank 3  set shape "temple3"  ]
    if population >= rank-2-pop [  set rank 2  set shape "temple2"  ]
    if population >= rank-1-pop [  set rank 1  set shape "temple1"  ]
    if start-rank < rank [  if count my-links > 0 [ask one-of my-links [die]]]] ; lose a road if drop a rank
  
  ask settlements with [ (rank = 3 and count link-neighbors <= 1 )   ] [ ; #
    let others other settlements with [link-neighbor? myself = false and (sqrt area * (distance myself)) <= 31]
    ifelse count others > 0 [  create-link-with max-one-of others [population]   ][]  ]
  ask settlements with [ ( rank = 2 and count link-neighbors <= 2 )  ] [ ; #
    let others other settlements with [link-neighbor? myself = false and (sqrt area * (distance myself)) <= 31 * (rank-2-pop / rank-3-pop / 2 + 1) ]
    ifelse count others > 0 [  create-link-with max-one-of others [population]   ][]  ]
  ask settlements with [ ( rank = 1 and count link-neighbors <= 3 )  ] [ ; #
    let others other settlements with [link-neighbor? myself = false and (sqrt area * (distance myself)) <= 31 * (rank-1-pop / rank-3-pop / 2  + 1) ]
    ifelse count others > 0 [  create-link-with max-one-of others [population]   ][]  ]
end

to trading
  ask settlements [ set trade-strength 0 set centrality 0 set cluster-number 0 ]
  ask links [set trade-flow 0]
  set traders (settlements with [ count link-neighbors > 0 ])
  find-all-components
  while [count traders with [ cluster-number = 0] > 0] [
    ask traders with [ cluster-number = 0] [
      set cluster-number max [cluster-number] of link-neighbors
      ask link-neighbors [ set cluster-number max [cluster-number] of link-neighbors     ] ]  ]
  ask traders [
    set visited-nodes []
    set centrality 0
    ask searchers [ die ]
    set search-completed false
    set origin-city one-of settlements-here
    hatch-searchers 1  [
      set size 1
      set color red
      set visited-nodes fput  one-of settlements-here  visited-nodes
      set path-cost [city-travel-cost] of one-of settlements-here
      set location one-of settlements-here  ]
    loop [
      ask origin-city [set centrality centrality + 1]
      ask searchers  [
        expand-paths (searcher who);;;; fixed
        die ]
      ifelse any? searchers   [ ][  set search-completed true  ]
      if search-completed = true [ stop]]]
  ask settlements [ set trade-strength 0]
  let mean-city-travel-cost mean [city-travel-cost] of traders
  ask traders [
    set trade-strength   (  ((1 + (( cluster-number  * 1)    /   (  centrality * 1  ))) ^ 0.9 ) ) / 30 ;/  city-travel-cost ^ 0.5 / 3   ;#
    if trade-strength < 0 [ set trade-strength 0]
    if trade-strength > 1 [ set trade-strength 1]]
end

to find-all-components
  ask traders [ set explored? false set cluster-number 0 ]
  loop  [
    set network-start one-of traders with [ not explored? ]
    if network-start = nobody [ stop ]
    set component-size 0
    ask network-start [ explore (gray + 2) ] ]
end

to explore [new-color]
  if explored? [ stop ]
  set explored? true
  set component-size component-size + 1
  ask network-start [set cluster-number cluster-number + 1]
  ask link-neighbors [ explore new-color ]
end

;;;; start this is the problem part of the code here ;;;

; command is run by searcher-agent, same as passed as argument
to expand-paths [searcher-agent]
  foreach
    ; out-link-neighbors: all turtles that can be reached by this turtle through a link
    ; sorted list of out-link-neighbors of searcher-agent.location
    sort [out-link-neighbors with [count my-links >= 0]] of [location] of searcher-agent ; list
    ; but then the value doesnt get passed to the command?
    ; the previous source, location refers to the iterated variable, 
    ; so I guess it must be the same in this version.
    [expand-path searcher-agent location] ; command
end

; command is run by searcher-agent, same as passed as argument
; node is the target location to search
to expand-path [searcher-agent node]
  if not search-completed   [
    if not member? node visited-nodes [
      set visited-nodes fput node visited-nodes
      if count searchers-here > 1 [die]
      ; why create a second searcher?
      hatch-searchers 1  [
        set size 1  set color red  set heading [heading] of searcher-agent  set visited-nodes [visited-nodes] of searcher-agent
        set path-cost [path-cost] of searcher-agent  set location node  move-to location  set path-cost path-cost + [city-travel-cost] of one-of settlements-here]]  ]
end

;to expand-paths [searcher-agent]

; sort [searcher-agent]
; sort searcher-agent ; link-neighbors with [count my-links >= 0]]

;end

;to expand-path
;  let node []
;  ask searcher [
;  if not search-completed   [
;      if not member? node visited-nodes [
;          set visited-nodes fput node visited-nodes
;          if count searchers-here > 1 [die]
;;          hatch-searchers 1  [
;           set size 1  set color red  set heading [heading] of searcher-agent  set visited-nodes [visited-nodes] of searcher-agent
;            set path-cost [path-cost] of searcher-agent  set location node  move-to location  set path-cost path-cost + [city-travel-cost] of one-of settlements-here]]  ]]
;end


;;;; end this is the problem part of the code here ;;;


to recalc-gdp
  ask settlements [
    set ecoserv-benefit (mean [ecosystem-services-value] of my-influence-patches )
    set ecoserv-GDP ecoserv-benefit * ecoserv-value
    if trade [
      ifelse count my-links > 0 [
        set trade-benefit trade-strength * trade-value
        ask my-links [
          set trade-flow trade-flow + [trade-benefit] of myself
          set thickness ( trade-flow ) / 10000   ]   ][ ; was 8k
        set trade-benefit 0  ] ]
    set trade-GDP trade-benefit
    set yield-GDP settlement-yield * ag-value
    set gdp-per-cap   ( yield-GDP + trade-GDP + ecoserv-GDP )  / population    ]
end

;##############         INTERFACE         ##################         INTERFACE        ##################

to update-view
  ask patches [set plabel ""]
  if View =  "Soil Degradation" [ set mx 400  set mn -5  ask land-patches [ set pcolor scale-color red ag-impact mx mn ]]
  if View =  "Population Gradient" [ set mx 10  set mn 0  ask land-patches [ set pcolor scale-color blue pop-gradient  mx mn  ]]
  if View =  "Temperature" [ set mx 30  set mn 8  ask land-patches [ set pcolor scale-color blue temp  mx mn  ]]
  if View =  "Soil Productivity" [ set mx  10 set mn 0  ask land-patches [ set pcolor scale-color brown soil-prod  mx mn  ]]
  if View =  "Elevation" [ set mx 1500  set mn -200  ask land-patches [ set pcolor scale-color gray elevation  mx mn  ]]
  if View =  "Precipitation" [ set mx  3000 set mn 0  ask land-patches [ set pcolor scale-color blue rainfall  mx mn  ]]
  if View =  "Ecosystem Services" [ set mx  200 set mn 0  ask land-patches [ set pcolor scale-color green ecosystem-services-value  mx mn  ]]
  if View =  "Net Primary Productivity" [ set mx 3000  set mn -5  ask land-patches [ set pcolor scale-color green npp  mx mn  ]]
  if View =  "Agricultural Suitability" [ set mx  1000 set mn -5  ask land-patches [ set pcolor scale-color brown ag-suit  mx mn ]]
  if View =  "Benefit Cost of Agriculture" [ set mx  300 set mn  0 ask land-patches [ ifelse BCA-ag < 0 [set pcolor red + 2] [ set pcolor scale-color green BCA-ag  mx mn  ]]]
  if View =  "Water Flow" [ set mx  1.5 set mn  -0.15 ask land-patches [ set pcolor scale-color blue flow  mx mn  ]]
  if View =  "Slope" [ set mx  15  set mn  0 ask land-patches [ set pcolor scale-color gray slope  mx mn  ]]
  if View =  "Forest State" [ ask land-patches [ if forest-state = 1 [set pcolor yellow + 1]  if is-ag = 1 [set pcolor gray + 3.5]  if forest-state = 2 [set pcolor green + 2] if forest-state = 3 [set pcolor green - 0.5]]]
  if View =  "Blank" [ ask land-patches [ set pcolor white ]]
  if View =  "Travel Cost" [ set mx 60  set mn -10  ask land-patches [ set pcolor scale-color yellow travel-cost  mx mn ]]
  ;if View =  "Trade Strength" [ set mx 1  set mn 0  ask land-patches [ set pcolor white ]  foreach sort-by [[who] of ?1 > [who] of ?2] settlements [     ask ?[        ask my-influence-patches [set pcolor scale-color red [trade-strength] of myself mx mn ]] ]]  have to figure out previous ? list and what the question mark is poointing to
  if View =  "Imagery" [import-pcolors-rgb "Maya.png"]
  if legend-on [make-legend]
  ; if Influence-view[ if count settlements > 0  [   foreach sort-by [[who] of ?1 < [who] of ?2] settlements [     ask ?[        ask my-influence-patches [set pcolor gray + 1 ]]    ]] ] ; [color] of myself ;; have to figure out previous ? list and what the question mark is poointing to
  if Agric-view [ ask settlements [ ask my-ag-patches [   set pcolor grey + 1     ]]]
  ask links [ set color grey + 1   ]
  ask settlements [   set size log (population) 100 ]
end

;to make-movie
;  user-message "Enter name ending with .mov"  let path user-new-file  if not is-string? path [ stop ]
;  reset   movie-start path   while [  ticks <= 300 ] [   vid:start-recorder
;    run-model movie-grab-view ] movie-close
;end

to make-legend
  ask patch (max-pxcor * 0.95) (max-pycor * 0.96) [
    set pcolor [pcolor] of one-of land-patches  with-min[pcolor]
    ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself           ]]]]
    set plabel mx
    set plabel-color white ]
  ask patch (max-pxcor * 0.95) (max-pycor * 0.84) [
    set pcolor [pcolor] of one-of land-patches  with-max[pcolor]
    ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself           ]]]]
    set plabel mn
    set plabel-color black]
  ask patch (max-pxcor * 0.95) (max-pycor * 0.9) [
    set pcolor ([pcolor] of one-of land-patches  with-max [pcolor]  + [pcolor] of one-of land-patches  with-min[pcolor])  / 2
    ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself ask neighbors [ set pcolor [pcolor] of myself           ]]]]
    set plabel (mx + mn) / 2
    set plabel-color white]
end

;    MayaSim: An agent-based model of the ancient Maya social-ecological system.
;    Copyright (C) 2013  Scott Heckbert
;
;    scott.heckbert@gmail.com
;
;    This program is free software: you can redistribute it and/or modify
;    it under the terms of the GNU Affero General Public License as published by
;    the Free Software Foundation, version 3.
;
;    This program is distributed in the hope that it will be useful,
;    but WITHOUT ANY WARRANTY; without even the implied warranty of
;    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;    GNU Affero General Public License for more details.