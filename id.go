package main

type ID struct {
	class, number, state int
}

//ID CLASSES
const (
	ITEMID = iota
	ACTORID
	TILEID
)

//Tile IDS
var (
	STONE_WALL  ID = ID{TILEID, 0, 0}
	STONE_FLOOR ID = ID{TILEID, 1, 0}
)

//Actor IDS
var (
	PLAYER ID = ID{ACTORID, 0, 0}
	DUMMY  ID = ID{ACTORID, 1, 0}
)
