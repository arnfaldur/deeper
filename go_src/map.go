package main

import (
	"math/rand"
)

var isSolid = map[ID]bool{
	STONE_WALL:  true,
	STONE_FLOOR: false,
}

func (m *Mapt) locateNPCs(npcs []NPC) {
	for i := range m {
		for j := range m[i] {
			m[i][j].npcsOnTile = nil
		}
	}
	for i, n := range npcs {
		m[int(imag(n.pos)+0.5)][int(real(n.pos)+0.5)].npcsOnTile = append(m[int(imag(n.pos)+0.5)][int(real(n.pos)+0.5)].npcsOnTile, &npcs[i])
	}
}

type Tile struct {
	Entity
	npcsOnTile []*NPC
}

func NewTile(name string, x float64, y float64) Tile {
	tile := AssMan.metaTiles[name]
	if tile.variations > 1 {
		tile.id.state = rand.Int() % tile.variations
	}
	tile.pos = complex(x, y)
	return tile.Tile
}
