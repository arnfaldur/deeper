package main

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
