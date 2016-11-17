package main

var IS_SOLID = map[ID]bool{
	STONE_WALL:  true,
	STONE_FLOOR: false,
}

type Tile struct {
	tileID, itemID ID
}
