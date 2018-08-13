package main

import (
	"math"
	"math/cmplx"
	"math/rand"
)

type Entity struct {
	//Runtime computable members
	id  ID
	pos complex128
	vel complex128
	//Load-time members
	Name      string
	Collision bool
	Solid     bool
	Size      float64
	Weight    float64
}

type Character struct {
	Entity
	MaxHealth, CurrHealth int
	attributes            [7]int
	damage                int
}

type Player struct {
	Character
}

type NPC struct {
	Character
	Aggro bool
}

func NewNPC(name string, x float64, y float64) NPC {
	npc := AssMan.metaCharacters[name]
	if npc.variations > 1 {
		npc.id.state = rand.Int() % npc.variations
	}
	npc.pos = complex(x, y)
	return npc.NPC
}

func (v *Character) attackBy(a Character) {
	v.CurrHealth -= a.damage
}

func getNPCsNear(pos complex128, radius float64) []*NPC {
	var npcs []*NPC
	for _, t := range vicinity(pos, radius) {
		if t[0] >= 0 && t[0] < MAPSIZE && t[1] >= 0 && t[1] < MAPSIZE {
			for i := range theMap[t[0]][t[1]].npcsOnTile {
				npcs = append(npcs, theMap[t[0]][t[1]].npcsOnTile[i])
			}
		}
	}
	return npcs
}

func (e *Entity) entityCollision() {

}

func (e *Entity) tileCollide(theMap *Mapt) {
	px, py := parts(e.pos + e.vel)

	pxf, pxr, pxc, pyf, pyr, pyc := int(px), int(px+0.5), int(math.Nextafter(px+1, math.Inf(-1))), int(py), int(py+0.5), int(math.Nextafter(py+1, math.Inf(-1)))
	any := false
	toWall := e.Size / 2

	//fs := [6]float64{py - 0.5, py, py + 0.5, px - 0.5, px, px + 0.5}
	//is := [6]int{pyf, pyr, pyc, pxf, pxr, pxc}
	//for _, i := range [4][2]int{{0,4},{2,4},{1,3},{1,5}} {
	//	colDir := complex(fs[i[1]]-float64(is[i[1]]), fs[i[0]]-float64(is[i[0]]))
	//	if theMap[is[i[0]]][is[i[1]]].Collision && toWall > cmplx.Abs(colDir) {
	//		colDep := toWall - cmplx.Abs(colDir)
	//		e.vel += cmplx.Rect(colDep, cmplx.Phase(colDir))
	//		e.pos += cmplx.Rect(colDep, cmplx.Phase(colDir))
	//	}
	//}

	if theMap[pyf][pxr].Collision && toWall >= math.Abs(py-0.5-float64(pyf)) {
		e.vel = complex(real(e.vel), math.Max(0, imag(e.vel)))
		e.pos = complex(real(e.pos), float64(pyf)+0.5+toWall)
		any = true
	} else if theMap[pyc][pxr].Collision && toWall >= math.Abs(py+0.5-float64(pyc)) {
		e.vel = complex(real(e.vel), math.Min(0, imag(e.vel)))
		e.pos = complex(real(e.pos), float64(pyc)-0.5-toWall)
		any = true
	}
	if theMap[pyr][pxf].Collision && toWall >= math.Abs(px-0.5-float64(pxf)) {
		e.vel = complex(math.Max(0, real(e.vel)), imag(e.vel))
		e.pos = complex(float64(pxf)+0.5+toWall, imag(e.pos))
		any = true
	} else if theMap[pyr][pxc].Collision && toWall >= math.Abs(px+0.5-float64(pxc)) {
		e.vel = complex(math.Min(0, real(e.vel)), imag(e.vel))
		e.pos = complex(float64(pxc)-0.5-toWall, imag(e.pos))
		any = true
	}
	if !any {
		fs := [4]float64{py - 0.5, py + 0.5, px - 0.5, px + 0.5}
		is := [4]int{pyf, pyc, pxf, pxc}
		for y := 0; y < 2; y++ {
			for x := 2; x < 4; x++ {
				colDir := complex(fs[x]-float64(is[x]), fs[y]-float64(is[y]))
				if theMap[is[y]][is[x]].Collision && toWall > cmplx.Abs(colDir) {
					colDep := toWall - cmplx.Abs(colDir)
					e.vel += cmplx.Rect(colDep, cmplx.Phase(colDir))
					e.pos += cmplx.Rect(colDep, cmplx.Phase(colDir))
					break
				}
			}
		}
	}
}

func (c *Character) npcCollide(npcs *[]NPC) {
	netForce := 0 + 0i
	neighbours := getNPCsNear(c.pos, (c.Size+MAXCHARSIZE+sqrt2)/2)
	for i, n := range neighbours {
		colDir := (c.pos + c.vel) - (n.pos + n.vel)
		colDep := (c.Size+n.Size)/2 - cmplx.Abs(colDir)
		if *c != n.Character && colDep > 0 {
			nudge := cmplx.Rect(colDep/2, cmplx.Phase(colDir))
			if *c == hilbert.Character {
				//neighbours[i].CurrHealth -= c.damage
				neighbours[i].vel -= cmplx.Rect(1, cmplx.Phase(colDir))
			}
			netForce += nudge

		}
	}
	c.vel += netForce
}

func (e *Entity) findCollisions(theMap Mapt) {
	for _, t := range vicinity(e.pos, e.Size+MAXCHARSIZE) {
		for _, n := range theMap[t[0]][t[1]].npcsOnTile {
			if cmplx.Abs(e.pos+e.vel-n.pos+n.vel) <= (e.Size+n.Size)/2 {

			}
		}
	}
}

func findAllCollisions(theMap Mapt, p Player, npcs []NPC) {
	p.findCollisions(theMap)
	for _, n := range npcs {
		n.findCollisions(theMap)
	}
}

func dealWithCollisions(theMap *Mapt, p *Player, npcs []NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = cNorm(moveDirection)
	}
	p.vel = approach(p.vel, moveDirection/5)
	for i, e := range npcs {
		if e.Aggro {
			diff := p.pos - e.pos
			npcs[i].vel = approach(e.vel, cNorm(diff)/10)
		}
	}
	p.npcCollide(&npcs)
	for i := range npcs {
		npcs[i].npcCollide(&npcs)
	}
	p.tileCollide(theMap)
	for i := range npcs {
		npcs[i].tileCollide(theMap)
	}
	p.pos += cMul(p.vel, timeDilation)
	for i := range npcs {
		npcs[i].pos += cMul(npcs[i].vel, timeDilation)
	}
}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{id: makeActorID(id), pos: pos, Name: "TestEnemy", Size: 0.8}, MaxHealth: 10, CurrHealth: 10}, true}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{id: DUMMY, pos: complex(float64(x), float64(y)), Name: "dummy"}, MaxHealth: 10, CurrHealth: 10}, true}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
