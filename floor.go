package main

type TILEID int

var IS_SOLID = map[TILEID]bool{
	STONE_WALL:  true,
	STONE_FLOOR: false,
}

//TileIDS
const (
	STONE_WALL TILEID = iota
	STONE_FLOOR
)

type Tile struct {
	tileID TILEID
	itemID int
}
