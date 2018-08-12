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
	for _, n := range npcs {
		m[int(imag(n.pos))][int(real(n.pos))].npcsOnTile = append(m[int(imag(n.pos))][int(real(n.pos))].npcsOnTile, &n)
	}
}
