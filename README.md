overview
========

simple client server terminal game about embarking on islands and messin' about.

how
---

developed with Rust and tested on linux only at this stage.

* server on 127.0.0.1:8080 with `cargo run server`
* client connecting to local only rn with `cargo run`

A good start with Rust → [The Rust Programming Language](https://doc.rust-lang.org/book/second-edition/index.html "The Rust Programming Language")

notable creates
---------------
`tui` for pretty terminal ui
`serde` for serializing data to send over tcp

realms ideas
============

scope
-----

* start of with an expedition of explorers on an island
* explorers have a class and can move independently
* positioning and actions as means to solve a puzzle (anticipation, preparation, resolve)
* final puzzle to complete island

puzzles
-------

* current situation leeds to event
* change current situation with explorers to prevent event

considerations
--------------

* move queue, advance with player input
* dropping and picking up gear
* particularities interaction
* explorer tracks region
* explorer specific modifications persistence on regions
* puzzle representation / non player entities for puzzles

plot
----

chapter I - the queen - affection

laying groundwork for the plot by shaping the state of the island and the relationship with the queen

* help the farmers
	+ reroute a river
	+ deliver goods
	+ build a well
	+ remove a blockade
* investigate
	+ look for specific particularities around the region
	+ combinations even
	+ report dialog
* fetch or deliver
	+ fetch information, goods, individuals
	+ deliver items, cargo, messages
* protect or escort
	+ move to region
	+ encounter on path
	+ path direct, disorientated or save and sound when information given
* conquer
    + remove a blockade
    + claim a region

what it looked like since this picture was last updated
=======================================================

![a screenshot of an early stage in development](screenshot.png "The client on the left with an overview of all regions, the current expedition explorer composition (with orders on the right) and the current region (selected or displaying the current explorers region with) with it's particularities. The server on the right shows connected clients (active ones are highlighted), created realms (just id for now) and every request.")

*The client on the left with an overview of all regions, the current expedition explorer composition (with orders on the right) and the current region (selected or displaying the current explorers region with) with it's particularities. The server on the right shows connected clients (active ones are highlighted), created realms (just id for now) and every request.*
