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
	STONE_WALL  = ID{TILEID, 0, 0}
	STONE_FLOOR = ID{TILEID, 1, 0}
)

//Actor IDS
var (
	PLAYER = ID{ACTORID, 0, 0}
	DUMMY  = ID{ACTORID, 1, 0}
)

func makeActorID(id int) ID {
	return ID{ACTORID, id, 0}
}
