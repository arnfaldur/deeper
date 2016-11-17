package main

//TileIDS
const (
	STONE_WALL  = iota
	STONE_FLOOR = iota
)

type Tile struct {
	tileID int
	itemID int
}
